// NEW

use crate::pci::devices::{
    Device,
    CommonHeader,
    Generic as PciGeneric
};

pub fn is_ahci_device(device: &dyn CommonHeader) -> bool
{
    device.get_header_type().get_type() == 0x0
    && device.get_class() == 0x01
    && device.get_subclass() == 0x06
    && device.get_programming_interface() == 0x01
}

pub fn init_device(device: &PciGeneric)
{
    debug_assert!(
        device.get_class() == 0x01
        && device.get_subclass() == 0x06
        && device.get_programming_interface() == 0x01);

    let addr = device.get_bar_5();
    let size = device.get_bar_5_size();
    println!("{:?} {}", addr, size);
}