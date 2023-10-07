// NEW

use core::fmt::{
    Result,
    Formatter,
    Display,
    Binary,
    LowerHex,
    UpperHex
};

use crate::drivers::Register;

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptStatus(Register<u32>);
impl InterruptStatus
{
    pub fn from_raw(value: u32) -> Self
    {
        Self(Register::new(value))
    }

    pub fn get_raw(&self) -> u32
    {
        self.0.get()
    }

    /// Clears all the bits, which are not resereved and are cleared through a write of 1.
    /// 
    /// Notable exception (unexhaustive list): PhyRdy Change Status (PRCS): Cleared by `PxSERR.DIAG.N`
    pub fn clear_all(&mut self)
    {
        // All clearable (Type RWC) bits are 1
        self.0.set(0xfd_80_00_afu32);
    }

    /// Cold Port Detect Status
    pub fn get_cpds(&self) -> bool
    {
        self.0.get() & (1u32 << 31) != 0
    }

    /// Cold Port Detect Status
    pub fn clear_cpds(&mut self)
    {
        // TODO: Test If Implementation is correct
        self.0.set(1u32 << 31);
    }

    /// Task File Error Status
    pub fn get_tfes(&self) -> bool
    {
        self.0.get() & (1u32 << 30) != 0
    }

    /// Task File Error Status
    pub fn clear_tfes(&mut self)
    {
        self.0.set(1u32 << 30);
    }

    /// Host Bus Fatal Error Status
    pub fn get_hbfs(&self) -> bool
    {
        self.0.get() & (1u32 << 29) != 0
    }

    /// Host Bus Fatal Error Status
    pub fn clear_hbfs(&mut self)
    {
        self.0.set(1u32 << 29);
    }

    /// Host Bus Data Error Status
    pub fn get_hbds(&self) -> bool
    {
        self.0.get() & (1u32 << 28) != 0
    }

    /// Host Bus Data Error Status
    pub fn clear_hdbs(&mut self)
    {
        self.0.set(1u32 << 28);
    }

    /// Interface Fatal Error Status
    pub fn get_ifs(&self) -> bool
    {
        self.0.get() & (1u32 << 27) != 0
    }

    /// Interface Fatal Error Status
    pub fn clear_ifs(&mut self)
    {
        self.0.set(1u32 << 27);
    }

    /// Interface Non-fatal Error Status
    pub fn get_infs(&self) -> bool
    {
        self.0.get() & (1u32 << 26) != 0
    }

    /// Interface Non-fatal Error Status
    pub fn clear_infs(&mut self)
    {
        self.0.set(1u32 << 26);
    }

    /// Overflow Status
    pub fn get_ofs(&self) -> bool
    {
        self.0.get() & (1u32 << 24) != 0
    }

    /// Overflow Status
    pub fn clear_ofs(&mut self)
    {
        self.0.set(1u32 << 24);
    }

    /// Incorrect Port Multiplier Status
    pub fn get_ipms(&self) -> bool
    {
        self.0.get() & (1u32 << 23) != 0
    }

    /// Incorrect Port Multiplier Status
    pub fn clear_imps(&mut self)
    {
        self.0.set(1u32 << 23);
    }

    /// PhyRdy Change Status (RO)
    pub fn get_prcs(&self) -> bool
    {
        self.0.get() & (1u32 << 22) != 0
    }

    /// Device Mechanical Presence Status
    pub fn get_dmps(&self) -> bool
    {
        self.0.get() & (1u32 << 7) != 0
    }

    /// Device Mechanical Presence Status
    pub fn clear_dmps(&mut self)
    {
        // Thinking about it:
        // Does this really need to be exposed?
        // The docs call it RWC (readable, clear by wrting 1)
        // But the desc makes it sound there is no reason for software to set it
        // I may implement functions which have 0 purpose.
        self.0.set(1u32 << 7);
    }

    /// Port Connect Change Status, RO
    pub fn get_pcs(&self) -> bool
    {
        self.0.get() & (1u32 << 6) != 0
    }

    /// Descriptor Processed
    pub fn get_dps(&self) -> bool
    {
        self.0.get() & (1u32 << 5) != 0
    }

    /// Descriptor Processed
    pub fn clear_dps(&mut self)
    {
        self.0.set(1u32 << 5);
    }

    /// Unknown FIS Interrupt, RO
    pub fn get_ufs(&self) -> bool
    {
        self.0.get() & (1u32 << 4) != 0
    }

    /// Set Device Bits Interrupt
    pub fn get_sdbs(&self) -> bool
    {
        self.0.get() & (1u32 << 3) != 0
    }

    /// Set Device Bits Interrupt
    pub fn clear_sdbs(&mut self)
    {
        self.0.set(1u32 << 3);
    }

    /// DMA Setup FIS Interrupt
    pub fn get_dss(&self) -> bool
    {
        self.0.get() & (1u32 << 2) != 0
    }

    /// DMA Setup FIS Interrupt
    pub fn clear_dss(&mut self)
    {
        self.0.set(1u32 << 2);
    }

    /// PIO Setup FIS Interrupt
    pub fn get_pss(&self) -> bool
    {
        self.0.get() & (1u32 << 1) != 0
    }

    /// PIO Setup FIS Interrupt
    pub fn clear_pss(&mut self)
    {
        self.0.set(1u32 << 1);
    }

    /// Device to Host Register FIS Interrupt
    pub fn get_dhrs(&self) -> bool
    {
        self.0.get() & 1u32 != 0
    }

    /// Device to Host Register FIS Interrupt
    pub fn clear_dhrs(&mut self)
    {
        self.0.set(1u32);
    }

}

impl Display for InterruptStatus
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
        if self.get_pcs()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Port Connect Change Status" } else { "pcs" })?;
        }
        if self.get_dps()
        {
            any = true;
            write!(f, "{}, ", if f.alternate() { "Descriptor Processed" } else { "dps" })?;
        }
        if self.get_ufs()
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

impl Binary for InterruptStatus
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

impl LowerHex for InterruptStatus
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

impl UpperHex for InterruptStatus
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
