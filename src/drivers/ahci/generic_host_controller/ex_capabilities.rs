// NEW

use core::fmt::{
    Display,
    Binary,
    LowerHex,
    UpperHex,
    Formatter,
    Result
};

use crate::drivers::util::Register;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitiesExtended(Register<u32>);
impl CapabilitiesExtended
{
    /// DevSleep Entrance from Slumber Only
    pub fn get_deso(&self) -> bool
    {
        self.0.get() & (1u32 << 5) != 0
    }

    /// Supports Aggressive Device Sleep Management
    pub fn get_sadm(&self) -> bool
    {
        self.0.get() & (1u32 << 4) != 0
    }

    /// Supports Device Sleep
    pub fn get_sds(&self) -> bool
    {
        self.0.get() & (1u32 << 3) != 0
    }

    /// Automatic Partial to Slumber Transitions
    pub fn get_apst(&self) -> bool
    {
        self.0.get() & (1u32 << 2) != 0
    }

    /// NVMHCI Present
    pub fn get_nvmp(&self) -> bool
    {
        self.0.get() & (1u32 << 1) != 0
    }

    /// BIOS/OS Handoff
    pub fn get_boh(&self) -> bool
    {
        self.0.get() & 1u32 != 0
    }
}

impl Display for CapabilitiesExtended
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        let mut any = false;
        if self.get_deso()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "DevSleep Entrance from Slumber Only" } else { "deso" })?;
        }
        if self.get_sadm()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Supports Aggressive Device Sleep Management" } else { "sadm" })?;
        }
        if self.get_sds()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Supports Device Sleep" } else { "sds" })?;
        }
        if self.get_apst()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Automatic Partial to Slumber Transitions" } else { "apst" })?;
        }
        if self.get_nvmp()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "NVMHCI Present" } else { "nvmp" })?;
        }
        if self.get_boh()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "BIOS/OS Handoff" } else { "boh" })?;
        }
        if !any
        {
            write!(f, "None")?;
        }
        Ok(())
    }
}

impl Binary for CapabilitiesExtended
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

impl LowerHex for CapabilitiesExtended
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

impl UpperHex for CapabilitiesExtended
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
