// NEW

mod interrupt_status;
use interrupt_status::*;

mod interrupt_enable;
use interrupt_enable::*;

mod command;
use command::*;

use crate::{
    arch::x86_64::{
        kernel::busy_sleep,
        mm::{
            paging::{
                map,
                PageTableEntryFlags,
                BasePageSize
            },
            physicalmem,
            virtualmem
        }
    },
    drivers::{
        Register,
        ahci::fis::{
            ReceivedFis,
            CommandHeader,
            CommandListStructure
        }
    }
};

use super::fis::CommandTable2;

pub struct Port
{
    pub virt_mem: usize,
    pub virt_fb: usize,
    pub virt_clb: usize,
    pub is_64bit_aware: bool,
    pub number_command_slots: u8
}

const TEMPLATE: Register<u16> = Register::new(0);

// Why u16? PRDT.DataByteCount must be a multiple of 2 (after decoding).
static mut BAD_IDEA: [Register<u16>; 256] = [TEMPLATE; 256];

static mut BAD_IDEA_2: usize = 0;

pub fn dump_bad_idea()
{
    unsafe {

        let bad_idea = &mut *(BAD_IDEA_2 as *mut [Register<u16>; 256]);

        for (i, it) in bad_idea.iter().enumerate()
        {
            if i % 8 == 0
            {
                println!();
            }
            print!("{:04x} ", it.get());
        }
        let sec0 = (bad_idea[100].get() as u64) | ((bad_idea[101].get() as u64) << 16) | ((bad_idea[102].get() as u64) << 32) | ((bad_idea[103].get() as u64) << 48);
        let sec1 = (bad_idea[60].get() as u64) | ((bad_idea[61].get() as u64) << 16);
        println!("\n{} MiB {} MiB", sec0 / 2048, sec1 / 2048);
    }
}

impl Port
{
    /*fn wait_for_detection(port: &PortRegister) -> bool
    {
        match port.ssts.get() & 0xf
        {
            0 | 4 => {
                busy_sleep(5000); // How long to wait is part of the SATA specs, which are not publically available. Therefor, I'm guessing.
            }
            1 | 3 => return true,
            _ => {}
        }

        match port.ssts.get() & 0xf
        {
            1 | 3 => true,
            0 => {
                #[cfg(debug_assertions)]
                println!("Port 404");
                false
            },
            4 => {
                #[cfg(debug_assertions)]
                println!("BIST loopback mode");
                false
            }
            it => {
                #[cfg(debug_assertions)]
                println!("Port in unknown status: {}", it);
                false
            }
        }
    }*/

    pub fn stop(port: &mut PortRegister) -> bool
    {
        // 10.1.2
        if port.cmd.get_st()
        {
            port.cmd.set_st(false);
        }
        // In theory, it might be set to false somewhere else, but still running
        for i in 0..4 // 4 tries
        {
            if port.cmd.get_cr()
            {
                busy_sleep(500);
            }
            else
            {
                break;
            }
        }
        if port.cmd.get_cr()
        {
            #[cfg(debug_assertions)]
            println!("Port Timeout waiting for PxCMD.CR");
            return false;
        }

        if port.cmd.get_fre()
        {
            port.cmd.set_fre(false);
        }
        for i in 0..4 // 4 tries
        {
            if port.cmd.get_fr()
            {
                busy_sleep(500);
            }
            else
            {
                break;
            }
        }
        if port.cmd.get_fr()
        {
            #[cfg(debug_assertions)]
            println!("Port Timeout waiting for PxCMD.FR");
            return false;
        }
        true
    }

    /// Returns: Virtual Addresses of (clb, fb)
    pub fn init(port: &mut PortRegister, is_64bit_aware: bool, num_command_slots: u8)
        -> (usize, usize)
    {
        // println!("  - Stopping");
        Self::stop(port);

        // We already did a full hba reset... do we try it again or do we fail with an assertion like it is now?
        assert!(!port.cmd.get_st());
        assert!(!port.cmd.get_cr());
        assert!(!port.cmd.get_fre());
        assert!(!port.cmd.get_fr());
        // bail on atapi? port.cmd.get_atapi

        // Why am I reallocating stuff?
        // Because I did not intend to expose a reserve function in the physical allocation code
        // and the addresses given by bios where inside "usable" space, which the frame allocater could happly give someone else.
        // (Some time passes) And because the docs in 10.1.2 specify this step (5.)

        // Without FIS-based Switching I can use one frame instead of 2.
        // If I have some time left, I could probably put multiple clb/fb into one frame, as they are together only 1280 bytes in size
        // which could result in 3 clb fb pairs in one frame.
        // Without fis based switching, CLB is more restrictive (CLB 1024 vs FB 256)
        // const ADDRESS_MASK: usize = (!0u32) as usize;
        // println!("  - Allocating");
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

        // println!("  - Init Mem");
        let new_vclb = new_page;
        let new_vfb = new_vclb + 1024usize;

        const CL_TEMPLATE: CommandHeader = CommandHeader::new();
        let cl_ptr = new_vclb as *mut CommandListStructure;
        unsafe { cl_ptr.write([CL_TEMPLATE; 32]) };

        const F_TEMPLATE: ReceivedFis = ReceivedFis::default();
        let f_ptr = new_vfb as *mut ReceivedFis;
        unsafe { f_ptr.write(F_TEMPLATE) };

        // Yes, write the PHYSICAL address, not virtual address.
        // println!("  - Set addresses");
        port.clb.set((new_clb & 0x00_00_00_00_ff_ff_ff_ffusize) as u32);
        port.clbu.set(((new_clb & 0xff_ff_ff_ff_00_00_00_00usize) >> 32) as u32);

        port.fb.set((new_fb & 0x00_00_00_00_ff_ff_ff_ffusize) as u32);
        port.fbu.set(((new_fb & 0xff_ff_ff_ff_00_00_00_00usize) >> 32) as u32);

        // println!("  - FRE");
        port.cmd.set_fre(true);

        // I did by error first implement the firmware steps (ups)
        // this seems to be a left over (wait_for_detection), but I could swear, I removed all of it
        // println!("  - Wait for detection");
        // Self::wait_for_detection(port);
        // clear all serial ata error and diagnostics
        port.serr.set(0x_ff_ff_ff_ff);
        port.is.clear_all();

        (new_vclb, new_vfb)

        // println!("CLB: {:#x} => {:#x} ({:#x})", clb, new_clb, ((port.clbu.get() as u64) << 32) | (port.clb.get() as u64));
        // println!("FB:  {:#x} => {:#x} ({:#x})", fb, new_fb, ((port.fbu.get() as u64) << 32) | (port.fb.get() as u64));
    }

    pub fn setup_interrupts(port: &mut PortRegister)
    {
        // Errors, some will always panic, some are pending a handler.
        port.ie.set_hbfs(true);
        port.ie.set_hdbs(true);
        port.ie.set_ifs(true);
        port.ie.set_infs(true);
        port.ie.set_ufe(true);
        port.ie.set_ipms(true);

        // A FIS was received
        port.ie.set_sdbs(true);
        port.ie.set_dss(true);
        port.ie.set_pss(true);
        port.ie.set_dhrs(true);
    }

    pub fn send_identify(port: &mut PortRegister, is_64bit_aware: bool, count_cmd_slots: u8, vclb: usize)
    {
        use super::fis::Fis;

        let dst = physicalmem::allocate(4096);
        let vdst = virtualmem::allocate(4096);

        if dst > 0xff_ff_ff_ff
        {
            virtualmem::deallocate(vdst, 4096);
            physicalmem::deallocate(dst, 4096);
            panic!("Address 64 bit, but hardware does not support 64 bit");
        }

        map::<BasePageSize>(vdst, dst, 1, PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITABLE | PageTableEntryFlags::WRITE_THROUGH);

        unsafe { (vdst as *mut u8).write_bytes(0, 4096) };

        let cmd_list = unsafe { &mut *(vclb as *mut CommandListStructure) };
        let cmd_slot = Self::find_free_cmd_slot(port, count_cmd_slots).expect("At least one slot free");
        let cmd_header = &mut cmd_list[cmd_slot as usize];

        cmd_header.reset();
        // unsafe { core::ptr::write_bytes(cmd_header, 0, 32) };

        cmd_header.set_prdtl(1);
        cmd_header.set_cfl(5);
        cmd_header.set_ctba(dst as u32);
        if is_64bit_aware
        {
            unsafe { cmd_header.set_ctbau(((dst as u64) >> 32) as u32) };
        }

        let ptr = core::ptr::from_raw_parts_mut::<CommandTable2>(vdst as *mut (), 1);
        unsafe {
            
            let cmd_table = &mut *ptr;
            cmd_table.zeroed();
            cmd_table.prdt[0].set(super::fis::PhysicalRegionDescriptorTable::new((dst as u64) + 2048, true, 512));

            let mut fis = super::fis::RegH2D::default();
            fis.pmport_cc.set(0x80);
            fis.command.set(0xEC); // ATA_CMD_IDENTIFY
            fis.countl.set(1);
            fis.copy_into(&mut cmd_table.cfis);
        };

        // REQUESTED or BUSY
        let mut fired = false;
        loop {
            if port.tfd.get() & (3 | 7) != 0
            {
                if !fired
                {
                    println!("Waiting");
                    fired = true;
                }
            }
            else
            {
                break;
            }
        }

        unsafe {

            BAD_IDEA_2 = vdst + 2048;
        }

        port.ci.set(1u32 << cmd_slot);

        println!("{:x}", port.tfd.get() & (3 | 7));
        busy_sleep(5000);
        loop {
            if port.tfd.get() & (3 | 7) == 0
            {
                break;
            }
        }
        println!("Should be done");
    }

    pub fn first_start(port: &mut PortRegister, is_64bit_aware: bool, count_cmd_slots: u8, vclb: usize, vfb: usize)
    {
        println!("FIRST START");
        port.cmd.set_st(true);
        let sig = port.sig.get();
        let sta = port.ssts.get();

        match sta & 0xf
        {
            0 => { println!("{:#x} No dice", sig); return; },
            1 => { println!("{:#x} Shy", sig); return; },
            3 => { println!("{:#x} Okay", sig); },
            4 => { println!("{:#x} Bist", sig); return; },
            _ => { println!("{:#x} UNKNOWN", sig); return; },
        }
        println!("  {:03x}", sta);

        Self::send_identify(port, is_64bit_aware, count_cmd_slots, vclb);

        // OSDevWiki, this FIS send to the device
        /* let mut fis = super::fis::RegH2D::default();
        fis.command.set(0xEC); // ATA_CMD_IDENTIFY
        // fis.pmport_cc.set(1); // pmport 0, c 1
        fis.pmport_cc.set(0x80); // Did I mix up the bits? Bit 0 should have been command/control
        fis.countl.set(1);

        // TODO
        println!("CMDSLOT!");
        let cmdslot = Self::find_free_cmd_slot(port, count_cmd_slots).expect("At least one command slot should be free.");

        // CLBU has the PHYSICAL ADDRESS, i need the VIRTUAL ADDRESS
        let command_list = unsafe { &mut *(vclb as *mut CommandListStructure) };
        let slot = &mut command_list[cmdslot as usize];
        slot.reset();
        slot.set_cfl(5);
        slot.set_clear_busy(true);
        slot.set_prdtl(1);
        // slot.set_write(false); // after zeroing out (= false), this is redundant

        // This is overkill, but I shot myself in the leg time wise.
        println!("ALLOCATE!");
        let phys = physicalmem::allocate(4096);
        let virt = virtualmem::allocate(4096);
        map::<BasePageSize>(
            virt,
            phys,
            1,
            PageTableEntryFlags::WRITABLE | PageTableEntryFlags::EXECUTE_DISABLE | PageTableEntryFlags::WRITE_THROUGH | PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITABLE);

        // CTBA must be a multiple of 128
        if phys > 0xff_ff_ff_ff && !is_64bit_aware
        {
            panic!("Physical address is 64 bit, but hardware does not support 64 bit");
        }

        println!("SETUP");
        // init the command table (copy our fis into it)
        // Metadata (second argument of from_raw_parts_mut) is the entries count of CommandTable2.PRDT (the slice at the end).
        let cmd_table = core::ptr::from_raw_parts_mut::<CommandTable2>(virt as *mut (), 1);
        unsafe {
            // PRDT is 1 length, therefore CommandTable2s size is 0x80 + 0x10 (length of single prdt)
            // Zero out the memory (valid bit pattern)
            core::ptr::write_bytes(cmd_table as *mut u8, 0u8, 0x80 + 0x10);

            // Init the set cfis data. All bytes, which are not of the fis (this fis is not 64 bytes long) are already set to 0.
            let cfis = core::ptr::addr_of_mut!((*cmd_table).cfis);
            core::ptr::copy_nonoverlapping(&fis as *const _ as *const u8, cfis as *mut u8, core::mem::size_of::<super::fis::RegH2D>());
            let tmp = &mut (*cmd_table).prdt[0];
            let bad_idea_phys = crate::arch::x86_64::mm::paging::get_physical_address::<BasePageSize>(&BAD_IDEA as *const _ as usize);
            tmp.set(crate::drivers::ahci::fis::PRDT::new(bad_idea_phys as u64, true, 512));
        }

        println!("ADDRESS!");
        slot.set_ctba(phys as u32);
        if phys > 0xff_ff_ff_ff
        {
            unsafe { slot.set_ctbau((phys >> 32) as u32) };
        }
        else if is_64bit_aware
        {
            unsafe { slot.set_ctbau(0) };
        }
        println!("SEND!");
        port.ci.set(1u32 << cmdslot);*/
    }

    fn find_free_cmd_slot(port: &PortRegister, count_cmd_slots: u8) -> Option<u8>
    {
        // SACT contains only Device Status (DS). The corresponding bit is set, before the bit is set in PxCI.
        let slots = port.sact.get() | port.ci.get();
        for i in 0..count_cmd_slots
        {
            // found slot
            if slots & (1u32 << i) == 0
            {
                return Some(i);
            }
        }
        None
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
