// NEW

use crate::drivers::Register;
use core::fmt::{
    Result,
    Formatter,
    // Display,
    Binary,
    UpperHex,
    LowerHex
};

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortsImplemented(Register<u32>);
impl PortsImplemented
{
    pub fn get(&self, idx: u8) -> bool
    {
        debug_assert!((0u8..=31u8).contains(&idx));
        self.0.get() & (1u32 << idx) != 0u32
    }
}

impl Binary for PortsImplemented
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        Binary::fmt(&self.0, f)
    }
}

impl UpperHex for PortsImplemented
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        UpperHex::fmt(&self.0, f)
    }
}

impl LowerHex for PortsImplemented
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        LowerHex::fmt(&self.0, f)
    }
}