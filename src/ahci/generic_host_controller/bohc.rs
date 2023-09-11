// NEW

use core::fmt::{
    Display,
    Binary,
    LowerHex,
    UpperHex,
    Formatter,
    Result
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiosOsHandoffControl(u32);
impl BiosOsHandoffControl
{
    pub fn from_raw(value: u32) -> Self
    {
        Self(value)
    }

    pub fn get_raw(self) -> u32
    {
        self.0
    }

    /// BIOS Busy
    pub fn get_bb(self) -> bool
    {
        self.0 & (1u32 << 4) != 0
    }

    /// BIOS Busy
    pub fn set_bb(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 4;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// BIOS Busy
    pub fn with_bb(mut self, value: bool) -> Self
    {
        self.set_bb(value);
        self
    }

    /// OS Ownership Change
    pub fn get_ooc(self) -> bool
    {
        self.0 & (1u32 << 3) != 0
    }

    /*
    /// OS Ownership Change
    pub fn set_ooc(&mut self)
    {
        todo!();
    }

    /// OS Ownership Change
    pub fn with_ooc(mut self) -> Self
    {
        self.set_ooc();
        self
    }*/

    /// SMI on OS Ownership Change Enable
    pub fn get_sooe(self) -> bool
    {
        self.0 & (1u32 << 2) != 0
    }

    /// SMI on OS Ownership Change Enable
    pub fn set_sooe(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 2;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// SMI on OS Ownership Change Enable
    pub fn with_sooe(mut self, value: bool) -> Self
    {
        self.set_sooe(value);
        self
    }

    /// OS Owned Semaphore
    pub fn get_oos(self) -> bool
    {
        self.0 & (1u32 << 1) != 0
    }

    /// OS Owned Semaphore
    pub fn set_oos(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 1;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// OS Owned Semaphore
    pub fn with_oos(mut self, value: bool) -> Self
    {
        self.set_oos(value);
        self
    }

    /// BIOS Owned Semaphore
    pub fn get_bos(self) -> bool
    {
        self.0 & 1u32 != 0
    }

    /// BIOS Owned Semaphore
    pub fn set_bos(&mut self, value: bool)
    {
        const MASK: u32 = 1u32;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// BIOS Owned Semaphore
    pub fn with_bos(mut self, value: bool) -> Self
    {
        self.set_bos(value);
        self
    }
}

impl Display for BiosOsHandoffControl
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        let mut any = false;
        if self.get_bb()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "BIOS Busy" } else { "bb" })?;
        }
        if self.get_ooc()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "OS Ownership Change" } else { "ooc" })?;
        }
        if self.get_sooe()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "SMI on OS Ownership Change Enable" } else { "sooe" })?;
        }
        if self.get_oos()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "OS Owned Semaphore" } else { "oos" })?;
        }
        if self.get_bos()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "BIOS Owned Semaphore" } else { "bos" })?;
        }
        if !any
        {
            write!(f, "None")?;
        }
        Ok(())
    }
}

impl Binary for BiosOsHandoffControl
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        if f.alternate()
        {
            write!(f, "{:#032b}", self.0)
        }
        else
        {
            write!(f, "{:032b}", self.0)
        }
    }
}

impl LowerHex for BiosOsHandoffControl
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        if f.alternate()
        {
            write!(f, "{:#08x}", self.0)
        }
        else
        {
            write!(f, "{:08x}", self.0)
        }
    }
}

impl UpperHex for BiosOsHandoffControl
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        if f.alternate()
        {
            write!(f, "{:#08X}", self.0)
        }
        else
        {
            write!(f, "{:08X}", self.0)
        }
    }
}
