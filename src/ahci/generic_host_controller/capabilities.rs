// NEW

use core::convert::TryInto;
use super::InterfaceSpeed;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities(u32);
impl Capabilities
{
    pub fn from_raw(value: u32) -> Self
    {
        Self(value)
    }

    pub fn get_raw(self) -> u32
    {
        self.0
    }

    /// Supports 64-bit Addressing
    pub fn get_s64a(self) -> bool
    {
        self.0 & (1u32 << 31) != 0
    }

    /// Supports Native Command Queuing
    pub fn get_sncq(self) -> bool
    {
        self.0 & (1u32 << 30) != 0
    }

    /// Supports SNotification Register
    pub fn get_ssntf(self) -> bool
    {
        self.0 & (1u32 << 29) != 0
    }

    /// Supports Mechanical Presence Switch
    pub fn get_smps(self) -> bool
    {
        self.0 & (1u32 << 28) != 0
    }

    /// Supports Staggered Spin-up
    pub fn get_sss(self) -> bool
    {
        self.0 & (1u32 << 27) != 0
    }

    /// Supports Aggressive Link Power Management
    pub fn get_salp(self) -> bool
    {
        self.0 & (1u32 << 26) != 0
    }

    /// Supports Activity LED
    pub fn get_sal(self) -> bool
    {
        self.0 & (1u32 << 25) != 0
    }

    /// Supports Command List Override
    pub fn get_sclo(self) -> bool
    {
        self.0 & (1u32 << 24) != 0
    }

    /// Interface Speed Support
    pub fn get_iss(self) -> InterfaceSpeed
    {
        let result = (self.0 & 0x00_f0_00_00) >> 20;
        if cfg!(debug_assertions)
        {
            InterfaceSpeed(result.try_into().expect("Expected the result to only be the lowest 4 bits, which should fit no problem into 8 bits"))
        }
        else
        {
            InterfaceSpeed(result as u8)
        }
    }

    // 19: Reserved

    /// Supports AHCI mode only
    pub fn get_sam(self) -> bool
    {
        self.0 & (1u32 << 18) != 0
    }

    /// Supports Port Multiplier
    pub fn get_spm(self) -> bool
    {
        self.0 & (1u32 << 17) != 0
    }

    /// FIS-based Switching Supported
    pub fn get_fbss(self) -> bool
    {
        self.0 & (1u32 << 16) != 0
    }

    /// PIO Multiple DRQ Block
    pub fn get_pmd(self) -> bool
    {
        self.0 & (1u32 << 15) != 0
    }

    /// Slumber State Capable
    pub fn get_ssc(self) -> bool
    {
        self.0 & (1u32 << 14) != 0
    }

    /// Partial State Capable
    pub fn get_psc(self) -> bool
    {
        self.0 & (1u32 << 13) != 0
    }

    /// Number of Command Slots per Port, as provided by the hardware
    /// 
    /// Note: returned value 0 means 1, returned value 1 means 2, ..., returned value 31 means 32
    pub fn get_ncs(self) -> u8
    {
        let result = (self.0 & 0x00_00_1f_00) >> 8;
        if cfg!(debug_assertions)
        {
            result.try_into().expect("Expected the result to only be the lowest 5 bits, which should fit no problem into 8 bits")
        }
        else
        {
            result as u8
        }
    }

    /// Number of Command Slots per Port, adjusted so 1 means 1, 32 means 32.
    pub fn get_ncs_adjusted(self) -> u8
    {
        self.get_ncs() + 1
    }

    /// Command Completion Coalescing Supported
    pub fn get_cccs(self) -> bool
    {
        self.0 & (1u32 << 7) != 0
    }

    /// Enclosure Management Supported
    pub fn get_ems(self) -> bool
    {
        self.0 & (1u32 << 6) != 0
    }

    /// Supports External SATA
    pub fn get_sxs(self) -> bool
    {
        self.0 & (1u32 << 5) != 0
    }

    /// Number of Ports, as provided by the hardware.
    /// 
    /// Actual Port Number may differ, this is only a theoretical supported number by the hba
    /// 
    /// Note: the value 0 means 1 port, the value 31 means 32 ports
    pub fn get_np(self) -> u8
    {
        (self.0 & 0x00_00_00_1f) as u8
    }

    /// Number of Ports, adjusted so 1 means 1, 32 means 32.
    pub fn get_np_adjusted(self) -> u8
    {
        self.get_np() + 1
    }
}

impl core::fmt::Display for Capabilities
{
    // TODO: Pretty Printing? Like a list of present capabilites?
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        let number_of_ports = self.get_np_adjusted();
        let number_of_command_slots = self.get_ncs_adjusted();
        let speed = self.get_iss();

        if self.get_sxs()
        {
            write!(f, "{}, ", if f.alternate() { "Supports External SATA" } else { "sxs" })?;
        }
        if self.get_ems()
        {
            write!(f, "{}, ", if f.alternate() { "Enclosure Management Supported" } else { "ems" })?;
        }
        if self.get_cccs()
        {
            write!(f, "{}, ", if f.alternate() { "Command Completion Coalescing Supported" } else { "cccs" })?;
        }
        if self.get_psc()
        {
            write!(f, "{}, ", if f.alternate() { "Partial State Capable" } else { "psc" })?;
        }
        if self.get_ssc()
        {
            write!(f, "{}, ", if f.alternate() { "Slumber State Capable" } else { "ssc" })?;
        }
        if self.get_pmd()
        {
            write!(f, "{}, ", if f.alternate() { "PIO Multiple DRQ Block" } else { "pmd" })?;
        }
        if self.get_fbss()
        {
            write!(f, "{}, ", if f.alternate() { "FIS-based Switching Supported" } else { "fbss" })?;
        }
        if self.get_spm()
        {
            write!(f, "{}, ", if f.alternate() { "Supports Port Multiplier" } else { "spm" })?;
        }
        if self.get_sam()
        {
            write!(f, "{}, ", if f.alternate() { "Supports AHCI mode only" } else { "sam" })?;
        }
        if self.get_sclo()
        {
            write!(f, "{}, ", if f.alternate() { "Supports Command List Override" } else { "sclo" })?;
        }
        if self.get_sal()
        {
            write!(f, "{}, ", if f.alternate() { "Supports Activity LED" } else { "sal" })?;
        }
        if self.get_salp()
        {
            write!(f, "{}, ", if f.alternate() { "Supports Aggressive Link Power Management" } else { "salp" })?;
        }
        if self.get_sss()
        {
            write!(f, "{}, ", if f.alternate() { "Supports Staggered Spin-up" } else { "sss" })?;
        }
        if self.get_smps()
        {
            write!(f, "{}, ", if f.alternate() { "Supports Mechanical Presence Switch" } else { "smps" })?;
        }
        if self.get_ssntf()
        {
            write!(f, "{}, ", if f.alternate() { "Supports SNotification Register" } else { "ssntf" })?;
        }
        if self.get_sncq()
        {
            write!(f, "{}, ", if f.alternate() { "Supports Native Command Queuing" } else { "sncq" })?;
        }
        if self.get_s64a()
        {
            write!(f, "{}, ", if f.alternate() { "Supports 64-bit Addressing" } else { "s64a" })?;
        }

        write!(f, "#CommandSlots: {}, #Ports: {}, Speed: {}", number_of_command_slots, number_of_ports, speed)
    }
}

impl core::fmt::Binary for Capabilities
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

impl core::fmt::LowerHex for Capabilities
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

impl core::fmt::UpperHex for Capabilities
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
