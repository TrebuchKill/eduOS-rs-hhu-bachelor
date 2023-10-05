// NEW, 4.2.3

use crate::drivers::Register;

use super::RegH2D;

/// From 0 to 65535 in the command table
#[repr(C)]
pub struct PhysicalRegionDescriptorTable
{
    dba: Register<u32>,
    dbau: Register<u32>,
    _reserved: Register<u32>,
    i_dbc: Register<u32>
}

pub type PRDT = PhysicalRegionDescriptorTable;

impl PhysicalRegionDescriptorTable
{
    pub const fn default() -> Self
    {
        Self {

            dba: Register::new(0),
            dbau: Register::new(0),
            _reserved: Register::new(0),
            i_dbc: Register::new(0)
        }
    }

    pub fn get_dba(&self) -> u32
    {
        self.dba.get()
    }

    pub fn set_dba(&mut self, value: u32)
    {
        debug_assert_eq!(value & 0xff_ff_ff_fe, value, "The address must be two byte aligned");
        self.dba.set(value);
    }

    pub fn get_dbau(&self) -> u32
    {
        self.dbau.get()
    }

    /// Unsafe Note: Only applicable for HBAs supporting 64 bit (HBA.CAP.S64A)
    pub unsafe fn set_dbau(&mut self, value: u32)
    {
        self.dbau.set(value);
    }

    pub fn get_interrupt_on_completion(&self) -> bool
    {
        (self.i_dbc.get() & 0x80_00_00_00) != 0
    }

    pub fn set_interrupt_on_completion(&mut self, value: bool)
    {
        self.i_dbc.set((self.i_dbc.get() & 0x7f_ff_ff_ff) | if value { 0x80_00_00_00 } else { 0 })
    }

    /// Data Byte Count
    pub fn get_dbc(&self) -> u32
    {
        self.i_dbc.get() & 0x00_3f_ff_ff
    }

    pub fn get_dbc_adjusted(&self) -> u32
    {
        self.get_dbc() + 1
    }

    /// Data Byte Count
    /// 
    /// Bit 0 must be set.
    /// 
    /// 1 gets interpreted as 2
    /// 3 gets interpreted as 4
    /// 5 gets interpreted as 6
    /// ...
    /// 4'194'303 get interpreted as 4'194'304 (4 MiB, max value)
    pub fn set_dbc(&mut self, value: u32)
    {
        debug_assert_eq!(value & 1, 1, "Bit 0 must be set!");
        self.i_dbc.set((self.i_dbc.get() & 0xff_c0_00_00) | value);
    }

    /// A Value of 0 becomes 1 and gets interpreted as 2
    /// 
    /// A value of 1 is illegal (bit 0 set)
    /// 
    /// A value of 2 becomes 3 and gets interpreted as 4
    /// 
    /// A value of 4 becomes 5 and gets interpreted as 6
    /// 
    /// A value of 4'194'302 becomes 4'194'303 and gets interpreted as 4'194'304 (4 MiB, max)
    /// 
    /// Thinking about it, this function makes no sense
    pub fn set_dbc_adjusted(&mut self, value: u32)
    {
        debug_assert_eq!(value & 1, 0, "The least significant bit shall be 0");
        debug_assert_eq!(value & 0x00_3f_ff_ff, value, "Only the 22 least significant bits can be set. Exception: Bit 0 must be 0");
        todo!("Abondend function as 'non-sense' before completion. May re-evaluate later.")
    }
}

// As defined right now, this struct would be more than 256 pages. That's bad.
// But I should have control over the entry count (needs verification.)
// Therefor I should have no problem (except maybe performance) to trim this structure down to something more managable
// 1048688 / 4096 ~ 256.027
pub struct CommandTable
{
    /// Command FIS (2 to 16 u32 in Length, must be a multiple of u32)
    /// 
    /// From 4.2.3.1: For Data Transfer Operations, this is the H2D Register FIS format as specified in the Serial ATA Revision 2.6 specification.
    cfis: [Register<u8>; 64],
    /// ATAPI Command (12 or 16 Bytes)
    acmd: [Register<u8>; 16],
    reserved: [Register<u8>; 0x30],
    prdt: [Register<PRDT>; 0xff_ff] // 65535 (ffffh) is the maximum count
}

/// Difference to CommandTable2: the last field (prdt) is defined as [Type], not as [Type; N] (Slice v Array?)
pub struct CommandTable2
{
    /// Command FIS (2 to 16 u32 in Length, must be a multiple of u32)
    /// 
    /// From 4.2.3.1: For Data Transfer Operations, this is the H2D Register FIS format as specified in the Serial ATA Revision 2.6 specification.
    cfis: [Register<u8>; 64],
    /// ATAPI Command (12 or 16 Bytes)
    acmd: [Register<u8>; 16],
    reserved: [Register<u8>; 0x30],
    prdt: [Register<PRDT>] // 65535 (ffffh) is the maximum count
}
