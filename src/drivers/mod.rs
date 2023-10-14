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
    ahci::on_interrupt(0);
}

pub fn on_interrupt(num: u8)
{
    ahci::on_interrupt(num);
}
