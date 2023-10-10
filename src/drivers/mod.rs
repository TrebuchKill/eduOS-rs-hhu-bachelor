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
    // loop {
    //     unsafe { x86::halt() };
    // }
    // crate::arch::x86_64::mm::physicalmem::debug_print();
}

pub fn on_interrupt(num: u8)
{
    ahci::on_interrupt(num);
}
