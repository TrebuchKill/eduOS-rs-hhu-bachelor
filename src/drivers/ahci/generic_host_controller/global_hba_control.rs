// NEW

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalHbaControl(u32);
impl GlobalHbaControl
{
    pub fn from_raw(value: u32) -> Self
    {
        Self(value)
    }

    pub fn get_raw(self) -> u32
    {
        self.0
    }

    /// AHCI Enable
    /// 
    /// If CAP.SAM is set to 1, this field is read only and be always 1
    /// 
    /// If CAP.SAM is set to 0, this field is read write and be reset to 0
    pub fn get_ae(self) -> bool
    {
        self.0 & (1u32 << 31) != 0
    }

    /// AHCI Enable
    /// 
    /// UNSAFE NOTE:
    /// 
    /// This is an illegal operation, when CAP.SAM is set.
    /// 
    /// This precondition is not checked by this function.
    /// 
    /// TODO: Writing 0 into this field needs to zero out these 32 bits
    pub unsafe fn with_ae(mut self, value: bool) -> Self
    {
        self.set_ae(value);
        self
    }

    /// AHCI Enable
    /// 
    /// UNSAFE NOTE:
    /// 
    /// This is an illegal operation, when CAP.SAM is set.
    /// 
    /// This precondition is not checked by this function.
    /// 
    /// TODO: Writing 0 into this field needs to zero out these 32 bits
    pub unsafe fn set_ae(&mut self, value: bool)
    {
        const SET_MASK: u32 = 1u32 << 31;
        if value
        {
            self.0 = self.0 | SET_MASK;
        }
        else
        {
            self.0 = self.0 & !SET_MASK;
        }
    }

    /// MSI Revert to Single Message
    pub fn get_mrsm(self) -> bool
    {
        self.0 & (1u32 << 2) != 0
    }

    /// Interrupt Enable
    pub fn get_ie(self) -> bool
    {
        self.0 & (1u32 << 1) != 0
    }

    /// Interrupt Enable
    pub fn with_ie(mut self, value: bool) -> Self
    {
        self.set_ie(value);
        self
    }

    /// Interrupt Enable
    pub fn set_ie(&mut self, value: bool)
    {
        const SET_MASK: u32 = 1u32 << 1;
        if value
        {
            self.0 |= SET_MASK;
        }
        else
        {
            self.0 &= !SET_MASK;
        }
    }

    /// HBA Reset
    pub fn get_hr(self) -> bool
    {
        self.0 & 1u32 != 0
    }

    /// HBA Reset
    /// 
    /// Always writes a true/1, as writing false/0 shall have no effect for the HBA and the HBA is supposed to reset this value to false/0 when it is done resetting.
    pub fn with_hr(mut self) -> Self
    {
        self.set_hr();
        self
    }

    /// HBA Reset
    /// 
    /// Always writes a true/1, as writing false/0 shall have no effect for the HBA and the HBA is supposed to reset this value to false/0 when it is done resetting.
    pub fn set_hr(&mut self)
    {
        self.0 |= 1;
    }
}

impl core::fmt::Display for GlobalHbaControl
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        let mut any = false;
        if self.get_ae()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "AHCI Enable" } else { "ae" })?;
        }
        if self.get_mrsm()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "MSI Revert to Single Message" } else { "mrsm" })?;
        }
        if self.get_ie()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Interrupt Enable" } else { "ie" })?;
        }
        if self.get_hr()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "HBA Reset" } else { "hr" })?;
        }
        if !any
        {
            write!(f, "None")?;
        }
        Ok(())
    }
}

impl core::fmt::Binary for GlobalHbaControl
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
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

impl core::fmt::LowerHex for GlobalHbaControl
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
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

impl core::fmt::UpperHex for GlobalHbaControl
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
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
