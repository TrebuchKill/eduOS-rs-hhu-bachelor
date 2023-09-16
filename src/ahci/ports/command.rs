use crate::ahci::macros::define_register;

use core::fmt::{
    Result,
    Formatter,
    Display,
    LowerHex,
    UpperHex,
    Binary
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IccState(u8);
impl IccState
{
    pub const DEVSLEEP: Self = Self(8);
    pub const SLUMBER: Self = Self(6);
    pub const PARTIAL: Self = Self(2);
    pub const ACTIVE: Self = Self(1);

    /// Alias: NOOP
    pub const IDLE: Self = Self(0);

    /// Alias: IDLE
    pub const NOOP: Self = Self(0);

    pub fn get_raw(self) -> u8
    {
        self.0
    }

    pub fn from_raw(value: u8) -> Option<Self>
    {
        if value < 0x10
        {
            Some(Self(value))
        }
        else
        {
            None
        }
    }
}

impl Display for IccState
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        write!(
            f,
            "{}",
            match *self
            {
                Self::DEVSLEEP => "DevSleep",
                Self::SLUMBER => "Slumber",
                Self::PARTIAL => "Partial",
                Self::ACTIVE => "Active",
                Self::IDLE => "Idle/NO-OP",
                _ => "Unknown value"
            })
    }
}

define_register!(
    struct Command;
    // Manuall ICC for now

    //May be read only
    rw 27 asp "Aggressive Slumber/Partial",
    rw 26 alpe "Aggressive Link Power Management Enable",
    rw 25 dlae "Drive LED on ATAPI Enable",
    rw 24 atapi "Device is ATAPI",
    rw 23 apste "Automatic Partial to Slumber Transitions Enable",
    ro 22 fbscp "FIS-based Switching Capable Port",
    ro 21 esp "External SATA Port",
    ro 20 cpd "Cold Presence Detection",
    ro 19 mpsp "Mechanical Presence Switch Attached to Port",
    ro 18 hpcp "Hot Plug Capable Port",

    // May be read only
    rw 17 pma "Port Multiplier Attached",
    ro 16 cps "Cold Presence State",
    ro 15 cr "Command List Running",
    ro 14 fr "FIS Receive Running",
    ro 13 mpss "Mechanical Presence Switch State",

    // Manuall CCS for now

    rw 4 fre "FIS Receive Enable",

    // The only rw1!
    rw1 3 clo "Command List Override",

    // May be read only
    rw 2 pod "Power On Device",
    
    // May be read only
    rw 1 sud "Spin-Up Device",
    rw 0 st "Start"
);

impl Command
{
    /// Interface Communication Control
    pub fn get_icc(self) -> IccState
    {
        IccState::from_raw(((self.0 & 0xf0_00_00_00u32) >> 28) as u8)
            .expect("Illegal value for PxCMD.ICC")
        /*match (self.0 & 0xf0_00_00_00u32) >> 28
        {
            0 => IccState::IDLE,
            1 => IccState::ACTIVE,
            2 => IccState::PARTIAL,
            6 => IccState::SLUMBER,
            8 => IccState::DEVSLEEP,
            i if i < 0x10 => IccState(i as u8),
            i => panic!("Illegal value for PxCMD.ICC: {}", i)
        }*/
    }

    /// Interface Communication Control
    pub fn set_icc(&mut self, value: IccState)
    {
        self.0 = (self.0 & 0x0f_ff_ff_ffu32) | ((value.0 as u32) << 28);
    }

    /// Current Command Slot
    pub fn get_ccs(self) -> u8
    {
        ((self.0 & 0x00_00_1f_00) >> 8) as u8
    }
}

impl Display for Command
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        let state = self.get_icc();
        let slot = self.get_ccs();

        if self.get_asp()
        {
            write!(f, "{}, ", if f.alternate() { "Aggressive Slumber/Partial" } else { "asp" })?;
        }
        if self.get_alpe()
        {
            write!(f, "{}, ", if f.alternate() { "Aggressive Link Power Management Enable" } else { "alpe" })?;
        }
        if self.get_dlae()
        {
            write!(f, "{}, ", if f.alternate() { "Drive LED on ATAPI Enable" } else { "dlae" })?;
        }
        if self.get_atapi()
        {
            write!(f, "{}, ", if f.alternate() { "Device is ATAPI" } else { "atapi" })?;
        }
        if self.get_apste()
        {
            write!(f, "{}, ", if f.alternate() { "Automatic Partial to Slumber Transitions Enable" } else { "apste" })?;
        }
        if self.get_fbscp()
        {
            write!(f, "{}, ", if f.alternate() { "FIS-based Switching Capable Port" } else { "fbscp" })?;
        }
        if self.get_esp()
        {
            write!(f, "{}, ", if f.alternate() { "External SATA Port" } else { "esp" })?;
        }
        if self.get_cpd()
        {
            write!(f, "{}, ", if f.alternate() { "Cold Presence Detection" } else { "cpd" })?;
        }
        if self.get_mpsp()
        {
            write!(f, "{}, ", if f.alternate() { "Mechanical Presence Switch Attached to Port" } else { "mpsp" })?;
        }
        if self.get_hpcp()
        {
            write!(f, "{}, ", if f.alternate() { "Hot Plug Capable Port" } else { "hpcp" })?;
        }
        if self.get_pma()
        {
            write!(f, "{}, ", if f.alternate() { "Port Multiplier Attached" } else { "pma" })?;
        }
        if self.get_cps()
        {
            write!(f, "{}, ", if f.alternate() { "Cold Presence State" } else { "cps" })?;
        }
        if self.get_cr()
        {
            write!(f, "{}, ", if f.alternate() { "Command List Running" } else { "cr" })?;
        }
        if self.get_fr()
        {
            write!(f, "{}, ", if f.alternate() { "FIS Receive Running" } else { "fr" })?;
        }
        if self.get_mpss()
        {
            write!(f, "{}, ", if f.alternate() { "Mechanical Presence Switch State" } else { "mpss" })?;
        }
        if self.get_fre()
        {
            write!(f, "{}, ", if f.alternate() { "FIS Receive Enable" } else { "fre" })?;
        }
        if self.get_clo()
        {
            write!(f, "{}, ", if f.alternate() { "Command List Override" } else { "clo" })?;
        }
        if self.get_pod()
        {
            write!(f, "{}, ", if f.alternate() { "Power On Device" } else { "pod" })?;
        }
        if self.get_sud()
        {
            write!(f, "{}, ", if f.alternate() { "Spin-Up Device" } else { "sud" })?;
        }
        if self.get_st()
        {
            write!(f, "{}, ", if f.alternate() { "Start" } else { "st" })?;
        }
        write!(f, "Slot: {}", slot)?;
        write!(f, "State: {}", state)
    }
}

impl Binary for Command
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

impl LowerHex for Command
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

impl UpperHex for Command
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
