// NEW

mod interrupt_status;
use interrupt_status::*;

mod interrupt_enable;
use interrupt_enable::*;

mod command;
use command::*;

use crate::drivers::Register;

pub struct Port
{
    pub hba: &'static super::AhciDevice,
    pub virt_fb: usize,
    pub virt_clb: usize,
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
