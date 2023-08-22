// NEW
mod generic;
mod pci_bridge;
mod card_bridge;

use super::DataType;

pub use generic::Generic;
pub use pci_bridge::PciBridge;
pub use card_bridge::CardBridge;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderType(u8);

impl HeaderType
{
    pub fn is_multifunction(&self) -> bool
    {
        self.0 & 0x80 == 0x80
    }

    pub fn get_type(&self) -> u8
    {
        self.0 & 0x7f
    }
}

pub trait Device
{
    fn get_bus(&self) -> u8;
    fn get_device(&self) -> u8;
    fn get_function(&self) -> u8;
}

pub trait CommonHeader
{
    fn get_vendor_id(&self) -> u16;
    fn get_device_id(&self) -> u16;
    fn get_command(&self) -> u16;
    fn get_status(&self) -> u16;
    fn get_revision_id(&self) -> u8;
    fn get_programming_interface(&self) -> u8;
    fn get_subclass(&self) -> u8;
    fn get_class(&self) -> u8;
    fn get_cache_line_size(&self) -> u8;
    fn get_latency_timer(&self) -> u8;
    fn get_header_type(&self) -> HeaderType;
    fn get_bist(&self) -> u8;
}

impl<T: Device> CommonHeader for T
{
    default fn get_vendor_id(&self) -> u16
    {
        u16::read(self.get_bus(), self.get_device(), self.get_function(), 0x00)
    }

    default fn get_device_id(&self) -> u16
    {
        u16::read(self.get_bus(), self.get_device(), self.get_function(), 0x02)
    }

    default fn get_command(&self) -> u16
    {
        u16::read(self.get_bus(), self.get_device(), self.get_function(), 0x04)
    }

    default fn get_status(&self) -> u16
    {
        u16::read(self.get_bus(), self.get_device(), self.get_function(), 0x06)
    }

    default fn get_revision_id(&self) -> u8
    {
        u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x08)
    }

    default fn get_programming_interface(&self) -> u8
    {
        u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x09)
    }

    default fn get_subclass(&self) -> u8
    {
        u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x0a)
    }

    default fn get_class(&self) -> u8
    {
        u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x0b)
    }

    default fn get_cache_line_size(&self) -> u8
    {
        u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x0c)
    }

    default fn get_latency_timer(&self) -> u8
    {
        u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x0d)
    }

    default fn get_header_type(&self) -> HeaderType
    {
        HeaderType(u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x0e))
    }

    default fn get_bist(&self) -> u8
    {
        u8::read(self.get_bus(), self.get_device(), self.get_function(), 0x0f)
    }
}

pub enum AnyDevice
{
    Generic(Generic),
    PciBridge(PciBridge),
    CardBridge(CardBridge)
}

impl AnyDevice
{
    pub(super) fn new(bus: u8, device: u8, function: u8) -> Option<Self>
    {
        if u16::read(bus, device, function, 0x0) == 0xff_ff
        {
            return None;
        }

        Some(
            match HeaderType(u8::read(bus, device, function, 0x0e)).get_type()
            {
                0 => AnyDevice::Generic(Generic { bus, device, function }),
                1 => AnyDevice::PciBridge(PciBridge { bus, device, function }),
                2 => AnyDevice::CardBridge(CardBridge { bus, device, function }),
                _ => unimplemented!() // May be short sighted?
            })
    }

    pub fn as_common_header(&self) -> &dyn CommonHeader
    {
        use AnyDevice::*;
        match self
        {
            Generic(it) => it,
            PciBridge(it) => it,
            CardBridge(it) => it
        }
    }
}
