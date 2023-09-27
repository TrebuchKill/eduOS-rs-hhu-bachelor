// NEW

mod macros;

mod generic_host_controller;
use generic_host_controller::*;

mod ports;
use ports::*;

mod fis;

use crate::{
    drivers::pci::{
        devices::{
            // Device as PciDevice,
            CommonHeader,
            Generic as PciGeneric,
            AnyDevice as AnyPciDevice
        },
        MemSpaceBarValue,
        // BarValue
    },
    arch::{
        mm::{
            paging::{
                PageTableEntryFlags,
                unmap,
                map
            },
            virtualmem
        },
        BasePageSize
    }, synch::spinlock::Spinlock
};

use core::convert::{
    TryFrom,
    TryInto
};

// Why don't I just use the paste crate?
// Do I want this macro really badley? Do I want to comprimise on the comment?
/*#[macro_export]
macro_rules! define_rwc
{
    ($getter:ident $setter:ident $pos:literal $($comment:literal)?) => {

        /// $comment
        pub fn $getter(self) -> bool
        {
            self.0 & (1u32 << $pos) != 0
        }

        /// $comment
        pub fn $setter(&mut self)
        {
            self.0 = 1u32 << $pos;
        }
    };
    ($($getter:ident $setter:ident $pos:literal $($comment:literal)?),+) => {

        $($crate::ahci::define_rwc!($getter $setter $pos $($comment)?);)+
    }
}*/

// https://stackoverflow.com/a/31749071
// pub use define_rwc;

pub struct AhciDevice
{
    device: PciGeneric,
    // abar: Option<&'a [u8]>
    abar_vaddr: usize,
    abar_actual_size: usize
}

impl AhciDevice
{
    // TODO
    // Currently this take ownership of PciGeneric (move)
    // Should I keep it that way?
    // Or should I switch to a clone/copy?
    pub fn new(device: PciGeneric) -> Option<Self>
    {
        if is_ahci_device(&device)
        {
            let mut ret = AhciDevice {
                device,
                abar_vaddr: 0,
                abar_actual_size: 0
            };
            ret.init();
            Some(ret)
        }
        else
        {
            None
        }
    }

    // Software may perform an HBA reset prior to initializing the controller by setting GHC.AE to
    // ‘1’ and then setting GHC.HR to ‘1’ if desired.
    // Page 104, Chap 10.1.2

    pub fn try_new(device: AnyPciDevice) -> Option<Self>
    {
        match device
        {
            AnyPciDevice::Generic(dev) => Self::new(dev),
            _ => None
        }
    }

    fn load_address(&mut self)
    {
        let addr = MemSpaceBarValue::try_from(self.device.get_bar_5()).unwrap();
        let size = self.device.get_bar_5_size();
        
        // Like the macro in src/macros.rs
        let size_page_aligned = (size + 0x0f_ff) & !0x0f_ffu32;

        #[cfg(debug_assertions)]
        println!("({:08x}, {:08x}, {:08x})", addr.address(), size, size_page_aligned);

        debug_assert_eq!(Ok(addr), self.device.get_bar_5().try_into());

        let vmem = virtualmem::allocate(size_page_aligned as usize);
        map::<BasePageSize>(
            vmem,
            addr.address() as usize,
            (size_page_aligned >> 12) as usize, // division by 0x10_00 is the same as right shift by 12, when the lowest 12 bits are 0, which they are, thanks to the alignment
            PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITABLE
        );
        self.abar_vaddr = vmem;
        self.abar_actual_size = size as usize;
        debug_assert_eq!(
            crate::arch::x86_64::mm::paging::get_page_table_entry::<BasePageSize>(vmem).map(|it| it.address()),
            Some(addr.address() as usize)
        );
    }

    // 10.6.3
    fn bios_os_handoff(&mut self)
    {
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        if mem.ghc.cap2.get_boh()
        {
            print!("BIOS OS Handoff");
            mem.ghc.bohc.set_oos(true);
            loop
            {
                print!(".");
                if !mem.ghc.bohc.get_bos()
                {
                    println!("!\nWait successful.");
                    break;
                }
            }
            todo!("Wait 25ms and check BOHC.BB"); // If set, wait at least 2 seconds for the bios tasks to complete
        }
    }

    // 10.4.3
    /// HBA Resets
    /// - GHC.AE
    /// - GHC.IE
    /// - IS Register
    /// - all port register fields except fields intiallized by hardware (HwInit) and PxFB/PxFBU/PxCLB/PxCLBU and 
    fn reset(&mut self)
    {
        let vaddr = self.abar_vaddr;
        let size = self.abar_actual_size;
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        println!("Resetting HBA...");
        println!(
            "{:#016x}\n{:#016x}\n{:#016x}",
            vaddr,
            size,
            mem as *mut _ as *const () as usize);
        mem.ghc.ghc.set_hr();
        print!("Waiting...");
        loop 
        {
            print!(".");
            // After a second consider HBA in locked/hung state
            if !mem.ghc.ghc.get_hr()
            {
                println!("!\nReset successful!");
                break;
            }
        }
    }

    fn enable_ahci_mode_and_interrupts(&mut self)
    {
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        
        // Enable ahci mode only if legacy mode is supported
        // Otherwise it should be already on and writing is undefined
        if !mem.ghc.ghc.get_ae() && !mem.ghc.cap.get_sam()
        {
            unsafe { mem.ghc.ghc.set_ae(true) }
        }

        // Enable interrupts
        mem.ghc.ghc.set_ie(true);
    }

    fn panic_on_yet_to_support(&self)
    {
        let mem = self.get_hba_mem().expect("Failed to load address");
        if mem.ghc.cap.get_sss()
        {
            todo!("Staggered Spin-up");
        }
        // TODO: PxCMD.CPD on any port
    }

    fn init_ports(&mut self)
    {
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        let number_of_command_slots = mem.ghc.cap.get_ncs_adjusted();
        let number_of_ports = mem.ghc.cap.get_np_adjusted();
        println!("NCS: {}, NP: {}", number_of_command_slots, number_of_ports);
        for i in 0u8..32u8
        {
            if mem.ghc.pi.get(i)
            {
                println!("Port {:2} implemented", i);
            }
        }
    }

    fn init(&mut self)
    {
        // Enable Inte
        let mut cmd = self.device.get_command();
        cmd.set_interrupt_disable(false);
        cmd.set_memory_space(true);
        cmd.set_bus_master(true);
        self.device.set_command(cmd);

        self.load_address();
        self.bios_os_handoff();
        self.reset();
        self.panic_on_yet_to_support();

        println!("TODO: Setup interrupts");
        self.enable_ahci_mode_and_interrupts();
        self.init_ports();

        /*let addr = MemSpaceBarValue::try_from(self.device.get_bar_5()).unwrap();
        let size = self.device.get_bar_5_size();
        // Like the macro in src/macros.rs
        let size_page_aligned = (size + 0x0f_ff) & !0x0f_ffu32;
        debug_assert_eq!(Ok(addr), self.device.get_bar_5().try_into());

        #[cfg(debug_assertions)]
        println!("({:08x}, {:08x}, {:08x})", addr.address(), size, size_page_aligned);

        let vmem = virtualmem::allocate(size_page_aligned as usize);
        map::<BasePageSize>(
            vmem,
            addr.address() as usize,
            (size_page_aligned >> 12) as usize, // division by 0x10_00 is the same as right shift by 12, when the lowest 12 bits are 0, which they are, thanks to the alignment
            PageTableEntryFlags::CACHE_DISABLE
        );
        self.abar_vaddr = vmem;
        self.abar_actual_size = size as usize;*/
    }

    fn calc_ports_slice_size(&self) -> usize
    {
        // Size of all the data
        // - the size of the data before the first port
        let tmp = self.abar_actual_size - 0x01_00;
        debug_assert_eq!(tmp % 0x80, 0);
        tmp / 0x80
    }

    fn get_hba_mem(&self) -> Option<&HbaMemory>
    {
        match self.abar_vaddr
        {
            0 => None,
            vmem => unsafe {

                Some(&*core::ptr::from_raw_parts(
                    vmem as *const (),
                    self.calc_ports_slice_size()))
            }
        }
    }

    fn get_hba_mem_mut(&mut self) -> Option<&mut HbaMemory>
    {
        match self.abar_vaddr
        {
            0 => None,
            vmem => unsafe {

                Some(&mut *core::ptr::from_raw_parts_mut(
                    vmem as *mut (),
                    self.calc_ports_slice_size()))
            }
        }
    }

    pub fn debug_print(&self)
    {
        let it = self.get_hba_mem().unwrap();
        /*let ptr = self.abar_vaddr;
        let it = unsafe { ptr.read_volatile() };*/
        println!("{}: {}", "cap", it.ghc.cap);
        println!("{}: {}", "ghc", it.ghc.ghc);
        println!("{}: {:x}", "is", it.ghc.is);
        println!("{}: {:032b}", "pi", it.ghc.pi);
        println!("{}: {:x}", "vs", it.ghc.vs);
        println!("{}: {:x}", "ccc_ctl", it.ghc.ccc_ctl);
        println!("{}: {:x}", "ccc_ports", it.ghc.ccc_ports);
        println!("{}: {:x}", "em_loc", it.ghc.em_loc);
        println!("{}: {:x}", "em_ctl", it.ghc.em_ctl);
        println!("{}: {}", "cap2", it.ghc.cap2);
        println!("{}: {}", "bohc", it.ghc.bohc);
        for i in 0..32
        {
            if it.ghc.pi.get(i)
            {
                let port = &it.ports[i as usize];
                println!("Port {:2}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
                    i,
                    "clb", port.value.clb,
                    "clbu", port.value.clbu,
                    "fb", port.value.fb,
                    "fbu", port.value.fbu,
                    "is", port.value.is,
                    "ie", port.value.ie,
                    "cmd", port.value.cmd,
                    "tfd", port.value.tfd,
                    "sig", port.value.sig,
                    "ssts", port.value.ssts,
                    "sctl", port.value.sctl,
                    "serr", port.value.serr
                );
            }
        }
    }
}

impl Drop for AhciDevice
{
    fn drop(&mut self)
    {
        let vmem = self.abar_vaddr;
        let size = self.abar_actual_size;
        if vmem != 0
        {
            self.abar_vaddr = 0;
            self.abar_actual_size = 0;

            let size_aligned = (size + 0x0f_ffusize) & !0x0f_ffusize;
            let count = size_aligned >> 12;
            unmap::<BasePageSize>(vmem, count);
            virtualmem::deallocate(vmem, size_aligned);
        }
    }
}

pub fn is_ahci_device<T>(device: &T) -> bool
    where T: CommonHeader + ?Sized
{
    device.get_header_type().get_type() == 0x0
    && device.get_class() == 0x01
    && device.get_subclass() == 0x06
    && device.get_programming_interface() == 0x01
}

// Maybe a struct, as an undefined value would be undefined behaviour with this code and with a struct could be correctly rejected as "unknown"?
// Frame Information Structure
// TODO: Redo with official specs
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

// HBA Mem Registers (all hex in bytes)
// 00..=2b Generic Host Control
// 2C..=FF Actually or effectivelly reserved (like vendor specific registers)



// According to VSCode, this struct has the intended layout (TODO: check this comment for typo)
// TODO: Makes this struct sense?
/// The thing, the ABAR (BAR\[5\]) points to
#[repr(C)]
pub struct HbaMemory
{
    /// Generic Host Control
    pub ghc: GenericHostControl,
    /// Reserved 0x2C..=0x5F
    /// 
    /// Reserved for NVMHCI 0x60..=0x9F
    /// 
    /// Vendor Specific 0xA0..=0xFF
    /// 
    /// These ranges are relative to the start of the HbaMemory struct, in this field, 0x2C should be at offset 0
    reserved: [u8; 0xd4],
    /// At least <TODO> ports must be present, at most 32 ports can be present.
    /// 
    /// Ports in the docs (and in this field) are numerated from 0 to 31
    pub ports: [AlignedPortRegisters]
}

static DEVICES: Spinlock<alloc::vec::Vec<AhciDevice>> = Spinlock::new(alloc::vec::Vec::new());

pub fn init()
{
    let mut devices = DEVICES.lock();
    if !devices.is_empty()
    {
        panic!("AHCI already initialized");
    }
    super::pci::on_each_generic_device_mut(|dev| {

        if let Some(dev) = AhciDevice::new(dev)
        {
            devices.push(dev);
        }
    })
}

pub fn on_each_device<F>(fun: F)
    where F: Fn(&AhciDevice) -> ()
{
    let devs = DEVICES.lock();
    for dev in &*devs
    {
        fun(dev);
    }
}

pub fn on_each_device_mut<F>(mut fun: F)
    where F: FnMut(&AhciDevice) -> ()
{
    let devs = DEVICES.lock();
    for dev in &*devs
    {
        fun(dev);
    }
}
