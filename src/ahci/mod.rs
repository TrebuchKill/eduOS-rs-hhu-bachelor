// NEW

use crate::pci::{devices::{
    Device,
    CommonHeader,
    Generic as PciGeneric
}, MemSpaceBarValue, BarValue};

use core::convert::{
    TryFrom,
    TryInto
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

    /*let addr = device.get_bar_5();
    let size = device.get_bar_5_size();
    println!("{:?} {}", addr, size);*/

    let mut cmd = device.get_command();
    println!("Original: {}", cmd);
    println!("Status: {}", device.get_status());
    cmd.set_interrupt_disable(true);
    cmd.set_memory_space(true);
    cmd.set_bus_master(true);
    println!("Edit: {}", cmd);
    device.set_command(cmd);
    let cmd = device.get_command();
    println!("Device: {}", cmd);
    println!("Status: {}", device.get_status());

    let addr = device.get_bar_5();
    let size_raw = device.get_bar_5_size();
    let size = {
        (size_raw + 0x0f_ff) & !0x0f_ffu32 // 0x10_00 = 4096 = "Basic Page Size"
    };
    debug_assert_eq!(addr, device.get_bar_5());

    let addr = MemSpaceBarValue::try_from(addr);
    println!("{:?} {:x} ({:x})", addr, size, size_raw);
    let addr = addr.unwrap();

    let vmem = crate::arch::mm::virtualmem::allocate(size as usize);
    crate::arch::mm::paging::map::<crate::arch::mm::paging::BasePageSize>(
        vmem,
        addr.address() as usize,
        1,
        crate::arch::mm::paging::PageTableEntryFlags::CACHE_DISABLE);
    println!("{:x}", vmem);

    // This code fails on my laptop
    // Looks like it does not need to be page aligned, the size that is
    // My laptop returns a size of 0x800 (2048) which is less then 0x1000 (4096)

    let vmem_ptr = vmem as *const HbaMem;
    let it = unsafe { vmem_ptr.read_volatile() };
    println!("{}: {:b}", "capability", it.capability);
    println!("{}: {:x}", "global_host_control", it.global_host_control);
    println!("{}: {:x}", "interrupt_status", it.interrupt_status);
    println!("{}: {:x}", "port_implemented", it.port_implemented);
    println!("{}: {:x}", "version", it.version);
    println!("{}: {:x}", "ccc_ctl", it.ccc_ctl);
    println!("{}: {:x}", "ccc_pts", it.ccc_pts);
    println!("{}: {:x}", "em_loc", it.em_loc);
    println!("{}: {:x}", "em_ctl", it.em_ctl);
    println!("{}: {:x}", "cap2", it.cap2);
    println!("{}: {:x}", "bohc", it.bohc);

    crate::arch::mm::paging::unmap::<crate::arch::mm::paging::BasePageSize>(vmem, 1);
    crate::arch::mm::virtualmem::deallocate(vmem, size as usize);
}

// Maybe a struct, as an undefined value would be undefined behaviour with this code and with a struct could be correctly rejected as "unknown"?
// Frame Information Structure
#[repr(u8)]
pub enum FisType
{
    RegisterHostToDevice = 0x27,
    RegisterDeviceToHost = 0x34,
    DmaActivateFis = 0x39,
    DmaSetup = 0x41,
    Data = 0x46,
    Bist = 0x58,
    PioSetup = 0x5f,
    DevBits = 0xa1
}

#[repr(C)]
pub struct HbaMem
{
    capability: u32,
    global_host_control: u32,
    interrupt_status: u32,
    port_implemented: u32,
    version: u32,
    ccc_ctl: u32,
    ccc_pts: u32,
    em_loc: u32,
    em_ctl: u32,
    cap2: u32,
    bohc: u32,
    _reserved: [u8; 0x74],
    vendor_specific: [u8; 0x60],
    // Following up to 32 ports
}

#[repr(C)]
pub struct HbaPort
{
}
