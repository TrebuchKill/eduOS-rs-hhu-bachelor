// NEW

//! WARNING: pmport is a bitfield. The osdev wiki entry does not specify the order of the bits, as it uses C/C++ structs, which can do whatever they want. Right now, redox-os wasn't a help either. This needs to be tested and fixed!

use crate::drivers::Register;

// This is part of the SATA Specs, which is not publicly available.
// This is fully reliant on https://wiki.osdev.org/AHCI and https://gitlab.redox-os.org/redox-os/drivers/-/blob/master/ahcid/src/ahci/fis.rs?ref_type=heads
// For the latter: Thanks to the wiki to use C code with bitfields (which can be in any order! https://stackoverflow.com/questions/19376426/order-of-fields-when-using-a-bit-field-in-c for C; N4860 (the C++20 standard) for C++ 11.4.9) making me either guess or use an existing implementation.
// I really hope I did not miss a note specifing the order :-D

// this enum (including comments) is straight up copied from https://wiki.osdev.org/AHCI#SATA_basic
// Maybe like InterfaceSpeed use a struct with constants instead?
#[repr(u8)]
#[allow(non_camel_case_types)] // I want to be close to the specs in this case
pub enum Type
{
    /// Register FIS - host to device
    REG_H2D = 0x27u8,

    /// Register FIS - device to host
    REG_D2H = 0x34u8,

    /// DMA activate FIS - device to host
    DMA_ACT = 0x39u8,

    /// DMA setup FIS - bidirectional
    DMA_SETUP = 0x41u8,

    /// Data FIS - bidirectional
    DATA = 0x46u8,

    /// BIST activate FIS - bidirectional
    BIST = 0x58u8,

    /// PIO setup FIS - device to host
    PIO_SETUP = 0x5fu8,

    /// Set device bits FIS - device to host
    DEV_BITS = 0xa1u8,
}

mod regh2d;
pub use regh2d::*;

mod regd2h;
pub use regd2h::*;

mod data;
pub use data::*;

mod pio_setup;
pub use pio_setup::*;

mod dma_setup;
pub use dma_setup::*;

mod command_list_structure;
pub use command_list_structure::*;

mod command_table;
pub use command_table::*;

/// The PxFB\[U\] points to memory containing this struct
/// 
/// PxFB must be 256 byte aligned (the size of the structure), with one exception.
/// 
/// If FIS-based switching is being used, it has to be 4096 byte aligned and the length will be extended to 4096.
/// 
/// FIS-based switching is currently unsupported.
#[repr(C)]
pub struct ReceivedFis
{
    pub dsfis: DmaSetup, // As I don't use packed, this is automatically padded to size 0x20, I do not need to insert the padding field like osdev wiki
    pub psfis: PioSetup,
    _psfis_pad: Register<[u8; 12]>,
    pub rfis: RegD2H,
    _rfis_pad: Register<[u8; 4]>,
    pub sdbfis: Register<[u8; 8]>, // No clue what FIS_DEV_BITS is, only the size of 8 bytes is documented by comment in the osdev wiki, in the docs "Set Device Bits FIS"
    /// Unknown FIS
    pub ufis: Register<[u8; 64]>,
    _reserved: Register<[u8; 0x60]>,
}

impl ReceivedFis
{
    pub const fn default() -> Self
    {
        Self {

            dsfis: DmaSetup::default(),
            psfis: PioSetup::default(),
            _psfis_pad: Register::new([0u8; 12]),
            rfis: RegD2H::default(),
            _rfis_pad: Register::new([0u8; 4]),
            sdbfis: Register::new([0u8; 8]),
            ufis: Register::new([0u8; 64]),
            _reserved: Register::new([0u8; 0x60])
        }
    }
}
