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

    /// data_byte_count must be 22 bits. The last bit is ignored, as the last bit has to be set.
    /// For Example: The value 0 and 1 will have the same result: Writing 1 and being interpreted as 2.
    pub fn new(dba: u64, interrupt_on_completion: bool, data_byte_count: u32) -> Self
    {
        assert_eq!(data_byte_count & 0x00_3f_ff_ff, data_byte_count);
        Self {

            dba: Register::new(dba as u32),
            dbau: Register::new(((dba & 0xff_ff_ff_ff_00_00_00_00u64) >> 32) as u32),
            _reserved: Register::new(0),
            i_dbc: Register::new(
                if interrupt_on_completion { 1u32 << 31 } else { 0 } | data_byte_count | 1
            )
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
    pub cfis: [Register<u8>; 64],
    /// ATAPI Command (12 or 16 Bytes)
    pub acmd: [Register<u8>; 16],
    reserved: [Register<u8>; 0x30],
    pub prdt: [Register<PRDT>; 0xff_ff] // 65535 (ffffh) is the maximum count
}

/// Difference to CommandTable2: the last field (prdt) is defined as [Type], not as [Type; N] (Slice v Array?)
pub struct CommandTable2
{
    /// Command FIS (2 to 16 u32 in Length, must be a multiple of u32)
    /// 
    /// From 4.2.3.1: For Data Transfer Operations, this is the H2D Register FIS format as specified in the Serial ATA Revision 2.6 specification.
    pub cfis: [Register<u8>; 64],
    /// ATAPI Command (12 or 16 Bytes)
    pub acmd: [Register<u8>; 16],
    pub reserved: [Register<u8>; 0x30],
    pub prdt: [Register<PRDT>] // 65535 (ffffh) is the maximum count
}

impl CommandTable2
{
    /// The count of PRDT Entries supported by AHCI.
    pub const MAX_PRDT_ENTRIES: u32 = 65535;

    /// The count of PRDT Entries fitting onto one page alongside the CommandTable
    pub const MAX_PRDT_ENTRIES_ON_ONE_PAGE: u32 = 248;

    pub fn zeroed(&mut self)
    {
        for it in &mut self.cfis
        {
            it.set(0);
        }
        for it in &mut self.acmd
        {
            it.set(0);
        }
        for it in &mut self.reserved
        {
            it.set(0);
        }
        for it in &mut self.prdt
        {
            it.set(PhysicalRegionDescriptorTable::default());
        }
    }
}

// Allocate and 0 out CommandTable
#[repr(transparent)]
pub struct CommandTable2Ptr(*mut CommandTable2);

impl CommandTable2Ptr
{
    pub const MAX_PRDT_ENTRIES: u32 = CommandTable2::MAX_PRDT_ENTRIES_ON_ONE_PAGE;

    pub fn new(num_prdt: u32) -> Self
    {
        use crate::arch::x86_64::mm::{
            paging::{
                self,
                BasePageSize,
                PageTableEntryFlags
            },
            physicalmem,
            virtualmem
        };

        // Per spec 65535 entires are allowed. But I have to limit it, as I want to fit it in one Page
        // One PRDT is 16 bytes. The base size (0 prdt) of CommandTable2 is 128. Padding in both included.
        // (4096 - 128) / 16 = 248
        // num_prdt can be at most 248 with my restriction, but in case I am off by one, I will check < 248.
        // I rather waste space (allow less entries) than write into another page (being one entry above).
        assert!(num_prdt > 0 && num_prdt < 248, "num_prdt must be between 0..<248");

        // Allocate PRDT memory 
        let pmem = physicalmem::allocate(4096);
        let vmem = virtualmem::allocate(4096);
        paging::map::<BasePageSize>(vmem, pmem, 1, PageTableEntryFlags::WRITABLE | PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITE_THROUGH);

        let prdt = core::ptr::from_raw_parts_mut(vmem as *mut (), num_prdt as usize);
        unsafe { (prdt as *mut u8).write_bytes(0, (128 + (num_prdt * 16)) as usize) };

        Self(prdt)
    }

    pub fn as_usize(&self) -> usize
    {
        self.0 as *const () as usize
    }

    pub fn as_u64(&self) -> u64
    {
        self.0 as *const () as u64
    }

    pub fn get_lo_u32(&self) -> u32
    {
        self.0 as *const () as u32
    }

    pub fn get_hi_u32(&self) -> u32
    {
        ((self.0 as *const () as u64) >> 32) as u32
    }

    pub fn as_ref(&self) -> &CommandTable2
    {
        unsafe { &*self.0 }
    }

    pub fn as_mut(&mut self) -> &mut CommandTable2
    {
        unsafe { &mut *self.0 }
    }
}

impl Drop for CommandTable2Ptr
{
    fn drop(&mut self)
    {
        use crate::arch::x86_64::mm::{
            paging::{
                self,
                BasePageSize,
                PageTableEntryFlags
            },
            physicalmem,
            virtualmem
        };
        let pmem = physicalmem::allocate(4096);
        let vmem = virtualmem::allocate(4096);
        paging::map::<BasePageSize>(vmem, pmem, 1, PageTableEntryFlags::WRITABLE | PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITE_THROUGH);
    }
}
