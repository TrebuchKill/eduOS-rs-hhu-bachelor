// NEW

use core::fmt::{
    Result,
    Formatter,
    Display,
    Binary,
    LowerHex,
    UpperHex
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptEnable(u32);
impl InterruptEnable
{
    pub fn from_raw(value: u32) -> Self
    {
        Self(value)
    }

    pub fn get_raw(self) -> u32
    {
        self.0
    }

    /// Cold Port Detect Status
    pub fn get_cpds(self) -> bool
    {
        self.0 & (1u32 << 31) != 0
    }

    /// Cold Port Detect Status
    /// 
    /// Unsafe Note: If cold presence is not supported, this field is read only, check PxCMD.CPD
    pub unsafe fn set_cpds(&mut self, value: bool)
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

    /// Task File Error Status
    pub fn get_tfes(self) -> bool
    {
        self.0 & (1u32 << 30) != 0
    }

    /// Task File Error Status
    pub fn set_tfes(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 30;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Host Bus Fatal Error Status
    pub fn get_hbfs(self) -> bool
    {
        self.0 & (1u32 << 29) != 0
    }

    /// Host Bus Fatal Error Status
    pub fn set_hbfs(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 29;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Host Bus Data Error Status
    pub fn get_hbds(self) -> bool
    {
        self.0 & (1u32 << 28) != 0
    }

    /// Host Bus Data Error Status
    pub fn set_hdbs(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 28;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Interface Fatal Error Status
    pub fn get_ifs(self) -> bool
    {
        self.0 & (1u32 << 27) != 0
    }

    /// Interface Fatal Error Status
    pub fn set_ifs(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 27;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Interface Non-fatal Error Status
    pub fn get_infs(self) -> bool
    {
        self.0 & (1u32 << 26) != 0
    }

    /// Interface Non-fatal Error Status
    pub fn set_infs(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 26;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Overflow Status
    pub fn get_ofs(self) -> bool
    {
        self.0 & (1u32 << 24) != 0
    }

    /// Overflow Status
    pub fn set_ofs(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 24;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Incorrect Port Multiplier Status
    pub fn get_ipms(self) -> bool
    {
        self.0 & (1u32 << 23) != 0
    }

    /// Incorrect Port Multiplier Status
    pub fn set_imps(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 23;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// PhyRdy Change Status (RO)
    pub fn get_prcs(self) -> bool
    {
        self.0 & (1u32 << 22) != 0
    }

    /// PhyRdy Change Interrupt Enable
    pub fn set_prce(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 22;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Device Mechanical Presence Status
    pub fn get_dmps(self) -> bool
    {
        self.0 & (1u32 << 7) != 0
    }

    /// Device Mechanical Presence Status
    /// 
    /// Unsafe Note: If Mechanical Presence Switch is not supported, this field is read only. See CAP.SMPS
    pub unsafe fn set_dmps(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 7;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Port Change Interrupt Enable
    pub fn get_pce(self) -> bool
    {
        self.0 & (1u32 << 6) != 0
    }

    /// Port Change Interrupt Enable
    pub fn set_pce(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 6;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Descriptor Processed
    pub fn get_dps(self) -> bool
    {
        self.0 & (1u32 << 5) != 0
    }

    /// Descriptor Processed
    pub fn set_dps(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 5;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// Unknown FIS Interrupt Enable
    pub fn get_ufe(self) -> bool
    {
        self.0 & (1u32 << 4) != 0
    }

    pub fn set_ufe(&mut self, value: bool)
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

    /// Set Device Bits Interrupt
    pub fn get_sdbs(self) -> bool
    {
        self.0 & (1u32 << 3) != 0
    }

    /// Set Device Bits Interrupt
    pub fn set_sdbs(&mut self, value: bool)
    {
        const MASK: u32 = 1u32 << 3;
        if value
        {
            self.0 |= MASK;
        }
        else
        {
            self.0 &= !MASK;
        }
    }

    /// DMA Setup FIS Interrupt
    pub fn get_dss(self) -> bool
    {
        self.0 & (1u32 << 2) != 0
    }

    /// DMA Setup FIS Interrupt
    pub fn set_dss(&mut self, value: bool)
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

    /// PIO Setup FIS Interrupt
    pub fn get_pss(self) -> bool
    {
        self.0 & (1u32 << 1) != 0
    }

    /// PIO Setup FIS Interrupt
    pub fn set_pss(&mut self, value: bool)
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

    /// Device to Host Register FIS Interrupt
    pub fn get_dhrs(self) -> bool
    {
        self.0 & 1u32 != 0
    }

    /// Device to Host Register FIS Interrupt
    pub fn set_dhrs(&mut self, value: bool)
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

}

impl Display for InterruptEnable
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        let mut any = false;
        if self.get_cpds()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Cold Port Detect Status" } else { "cpds" })?;
        }
        if self.get_tfes()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Task File Error Status" } else { "tfes" })?;
        }
        if self.get_hbfs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Host Bus Fatal Error Status" } else { "hbfs" })?;
        }
        if self.get_hbds()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Host Bus Data Error Status" } else { "hbds" })?;
        }
        if self.get_ifs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Interface Fatal Error Status" } else { "ifs" })?;
        }
        if self.get_infs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Interface Non-fatal Error Status" } else { "infs" })?;
        }
        if self.get_ofs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Overflow Status" } else { "ofs" })?;
        }
        if self.get_ipms()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Incorrect Port Multiplier Status" } else { "ipms" })?;
        }
        if self.get_prcs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "PhyRdy Change Status (RO)" } else { "prcs" })?;
        }
        if self.get_dmps()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Device Mechanical Presence Status" } else { "dmps" })?;
        }
        if self.get_pce()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Port Connect Change Status" } else { "pce" })?;
        }
        if self.get_dps()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Descriptor Processed" } else { "dps" })?;
        }
        if self.get_ufe()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Unknown FIS Interrupt" } else { "ufs" })?;
        }
        if self.get_sdbs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Set Device Bits Interrupt" } else { "sdbs" })?;
        }
        if self.get_dss()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "DMA Setup FIS Interrupt" } else { "dss" })?;
        }
        if self.get_pss()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "PIO Setup FIS Interrupt" } else { "pss" })?;
        }
        if self.get_dhrs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Device to Host Register FIS Interrupt" } else { "dhrs" })?;
        }

        if !any
        {
            write!(f, "None")
        }
        else
        {
            Ok(())
        }
    }
}

impl Binary for InterruptEnable
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

impl LowerHex for InterruptEnable
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

impl UpperHex for InterruptEnable
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
