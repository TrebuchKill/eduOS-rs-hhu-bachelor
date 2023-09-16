// NEW

mod interrupt_status;
use interrupt_status::*;

mod interrupt_enable;
use interrupt_enable::*;

mod command;
use command::*;

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
    pub clb: u32,
    /// Higher 32-bit Command List Base address, 0 and read only when 64 bit addressing not supported (s64a 0?)
    pub clbu: u32,

    /// lower 32-bit Fis Base address
    /// 
    /// When FIS based switching is...
    /// - off: 256 bytes aligned
    /// - on: 4096 bytes aligned
    pub fb: u32,
    /// higher 32-bit Fis Base address, 0 and read only when 64 bit addressing not supported (s64a 0?)
    pub fbu: u32,
    /// Interrupts Status
    pub is: InterruptStatus,
    /// Interrupt Enable, not Internet Explorer
    pub ie: InterruptEnable,
    /// Command and Status
    pub cmd: Command,
    /// reserved, like always, should be 0
    _reserved: u32,
    /// Task File Data
    pub tfd: u32,
    /// Signature
    pub sig: u32,
    /// Serial ata STatuS
    pub ssts: u32,
    /// Serial ata ConTroL
    pub sctl: u32,
    /// Serial ata ERRor
    pub serr: u32
}

// because I had no luck with #[repr(C, align(0x80))]
pub struct AlignedPortRegisters
{
    pub value: PortRegister,
    _padding: [u8; 0x4c]
}
