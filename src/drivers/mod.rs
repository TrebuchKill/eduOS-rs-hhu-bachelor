// NEW

pub mod util;
pub mod pci;
pub mod ahci;

pub fn init()
{
    pci::init();
    ahci::init();
}
