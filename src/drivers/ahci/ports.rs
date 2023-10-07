// NEW

mod interrupt_status;
use interrupt_status::*;

mod interrupt_enable;
use interrupt_enable::*;

mod command;
use command::*;

use crate::drivers::{
    Register,
    ahci::fis::{
        ReceivedFis,
        CommandHeader,
        CommandListStructure
    }
};

pub struct Port
{
    pub hba: &'static super::AhciDevice,
    pub virt_fb: usize,
    pub virt_clb: usize,
}

impl Port
{
    pub fn init(port: &mut PortRegister, is_64bit_aware: bool, port_idx: u8)
    {
        use crate::arch::x86_64::mm::{
            paging::{
                map,
                PageTableEntryFlags,
                BasePageSize
            },
            physicalmem,
            virtualmem
        };

        assert!(port_idx < 32);
        // bail on atapi? port.cmd.get_atapi

        port.cmd.get_st();

        let status = port.ssts.get();
        let sig = port.sig.get();
        let clb = ((port.clbu.get() as u64) << 32) | (port.clb.get() as u64);
        let fb = ((port.fbu.get() as u64) << 32) | (port.fb.get() as u64);
        // Why am I reallocating stuff?
        // Because I did not intend to expose a reserve function in the physical allocation code
        // and the addresses given by bios where inside "usable" space, which the frame allocater could happly give someone else.

        // Without FIS-based Switching I can use one frame instead of 2.
        // If I have some time left, I could probably put multiple clb/fb into one frame, as they are together only 1280 bytes in size
        // which could result in 3 clb fb pairs in one frame.
        // Without fis based switching, CLB is more restrictive (CLB 1024 vs FB 256)
        // const ADDRESS_MASK: usize = (!0u32) as usize;
        let new_frame = physicalmem::allocate(4096);
        if !is_64bit_aware && new_frame > 0xff_ff_ff_ffusize
        {
            physicalmem::deallocate(new_frame, 4096);
            panic!("Could not move memory (new address is 64 bit while the HBA does not support 64 bit).");
        }

        let new_clb = new_frame;
        let new_fb = new_clb + 1024usize; // CLB is 1024 bytes in size (no FIS-based Switching) and 1024 is a multiple of 256

        let new_page = virtualmem::allocate(4096);
        map::<BasePageSize>(new_page, new_frame, 1, PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITABLE | PageTableEntryFlags::WRITE_THROUGH);

        let new_vclb = new_page;
        let new_vfb = new_vclb + 1024usize;
        println!("Pre-Write");

        const CL_TEMPLATE: CommandHeader = CommandHeader::new();
        let cl_ptr = new_vclb as *mut CommandListStructure;
        unsafe { cl_ptr.write([CL_TEMPLATE; 32]) };

        const F_TEMPLATE: ReceivedFis = ReceivedFis::default();
        let f_ptr = new_vfb as *mut ReceivedFis;
        unsafe { f_ptr.write(F_TEMPLATE) };

        // Yes, write the PHYSICAL address, not virtual address.
        port.clb.set((new_clb & 0x00_00_00_00_ff_ff_ff_ffusize) as u32);
        port.clbu.set(((new_clb & 0xff_ff_ff_ff_00_00_00_00usize) >> 32) as u32);

        port.fb.set((new_fb & 0x00_00_00_00_ff_ff_ff_ffusize) as u32);
        port.fbu.set(((new_fb & 0xff_ff_ff_ff_00_00_00_00usize) >> 32) as u32);

        println!("CLB: {:#x} => {:#x} ({:#x})", clb, new_clb, ((port.clbu.get() as u64) << 32) | (port.clb.get() as u64));
        println!("FB:  {:#x} => {:#x} ({:#x})", fb, new_fb, ((port.fbu.get() as u64) << 32) | (port.fb.get() as u64));
        // let mut fis = fis::RegH2D::default();
        // fis.command.set(0xEC); // ATA_CMD_IDENTIFY
        // fis.pmport_cc.set(1); // pmport 0, c 1
    }
}

// Alignment is 4
// this fits into the 0x80 spacing between ports
// first port (0) starts at 0x100 (relative to the beginning of the HBA Memory Registers)
// second port (1) starts at 0x180
// the last potential port (31) starts at 0x1080
// if port 30 and therefor port 31 are present, as they start at 0x1000, more than one page will be required for mapping
// AHCI Spec 3.3
#[repr(C)]
pub struct PortRegister
{
    // 0xff_ff_fc_00
    /// Lower 32-bit Command List Base address, 1024 aligned (lowest 10 bits are always 0)
    /// 
    /// BUGBUG: the physical address, which is split between clbu and clb may be in a "usable" address and may need relocation. As it is exempt from a reset, maybe set it before a reset to a allocated frame, which is (overaligned to 4096)?
    pub clb: Register<u32>,
    /// Higher 32-bit Command List Base address, 0 and read only when 64 bit addressing not supported (s64a 0?)
    /// 
    /// BUGBUG: the physical address, which is split between clbu and clb may be in a "usable" address and may need relocation. As it is exempt from a reset, maybe set it before a reset to a allocated frame, which is (overaligned to 4096)?
    pub clbu: Register<u32>,

    /// lower 32-bit Fis Base address
    /// 
    /// When FIS based switching is...
    /// - off: 256 bytes aligned
    /// - on: 4096 bytes aligned
    /// 
    /// BUGBUG: the physical address, which is split between fbu and fb may be in a "usable" address and may need relocation. As it is exempt from a reset, maybe set it before a reset to a allocated frame, which is (overaligned to 4096)?
    pub fb: Register<u32>,
    /// higher 32-bit Fis Base address, 0 and read only when 64 bit addressing not supported (s64a 0?)
    /// 
    /// BUGBUG: the physical address, which is split between fbu and fb may be in a "usable" address and may need relocation. As it is exempt from a reset, maybe set it before a reset to a allocated frame, which is (overaligned to 4096)?
    pub fbu: Register<u32>,
    /// Interrupts Status
    pub is: InterruptStatus,
    /// Interrupt Enable, not Internet Explorer
    pub ie: InterruptEnable,
    /// Command and Status
    pub cmd: Command,
    /// reserved, like always, should be 0
    _reserved0: Register<u32>,
    /// Task File Data
    pub tfd: Register<u32>,
    /// Signature
    pub sig: Register<u32>,
    /// Serial ata STatuS
    pub ssts: Register<u32>,
    /// Serial ata ConTroL
    pub sctl: Register<u32>,
    /// Serial ata ERRor
    pub serr: Register<u32>,
    /// Serial ATA Active (SCR3: SActive)
    pub sact: Register<u32>,
    /// Command Issue
    pub ci: Register<u32>,
    /// Serial ATA Notification (SCR4: SNotification)
    pub sntf: Register<u32>,
    /// FIS-based Switching Control
    pub fbs: Register<u32>,
    /// Device Sleep
    pub devslp: Register<u32>,
    _reserved1: [Register<u32>; 10],
    /// Vendor Specific
    pub vs: [Register<u32>; 4]
}
