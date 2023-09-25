// NEW

pub mod util;
pub mod pci;
pub mod ahci;

// "Late" addition. Should I keep it?
pub use util::Register;

pub fn init()
{
    pci::init();
    ahci::init();
}
