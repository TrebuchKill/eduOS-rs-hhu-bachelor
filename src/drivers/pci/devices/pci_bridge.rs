// NEW

use super::Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciBridge
{
    pub(super) bus: u8,
    pub(super) device: u8,
    pub(super) function: u8
}

impl Device for PciBridge
{
    fn get_bus(&self) -> u8
    {
        self.bus
    }

    fn get_device(&self) -> u8
    {
        self.device
    }

    fn get_function(&self) -> u8
    {
        self.function
    }
}

impl core::fmt::Display for PciBridge
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        write!(f, "PCI({}, {}, {})", self.get_bus(), self.get_device(), self.get_function())
    }
}
