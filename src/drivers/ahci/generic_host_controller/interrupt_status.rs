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

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct InterruptStatus(Register<u32>);
impl InterruptStatus
{
    pub fn get(&self, port_index: u8) -> bool
    {
        debug_assert!((0u8..=31u8).contains(&port_index));
        self.0.get() & (1u32 << port_index) != 0u32
    }

    pub fn clear(&mut self, port_index: u8)
    {
        debug_assert!((0u8..=31u8).contains(&port_index));
        self.0.set(1u32 << port_index)
    }
}

impl Binary for InterruptStatus
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        Binary::fmt(&self.0, f)
    }
}

impl UpperHex for InterruptStatus
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        UpperHex::fmt(&self.0, f)
    }
}

impl LowerHex for InterruptStatus
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        LowerHex::fmt(&self.0, f)
    }
}
