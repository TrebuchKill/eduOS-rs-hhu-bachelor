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

        // #[cfg(debug_assertions)]
        // println!("({:08x}, {:08x}, {:08x})", addr.address(), size, size_page_aligned);

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
            #[cfg(debug_assertions)] // Maybe always?
            serial_println!("BIOS OS Handoff");
            mem.ghc.bohc.set_oos(true);
            loop
            {
                if !mem.ghc.bohc.get_bos()
                {
                    #[cfg(debug_assertions)] // Maybe always?
                    serial_println!("Wait for BOHC.bos successful");

                    crate::arch::x86_64::kernel::busy_sleep(30); // Doc says 25, but my code is not "perfect" 1 ms. Hope this will compensate enough.

                    let mut tries = 0u32;
                    while tries < 10 && mem.ghc.bohc.get_bb()
                    {
                        tries += 1;
                        crate::arch::x86_64::kernel::busy_sleep(2000); // Wait the minimum amount of time for the bios to clear this bit, at most 10 times (random choice).
                    }

                    if tries >= 10 && mem.ghc.bohc.get_bb() // after 10 tries of waiting 2 seconds, the bios busy flag is still set. I choose to timeout this operation at this point.
                    {
                        panic!("Bios OS Handoff failed: timeout");
                    }
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
        // let vaddr = self.abar_vaddr;
        // let size = self.abar_actual_size;
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        print!("Resetting HBA");
        /*println!(
            "{:#016x}\n{:#016x}\n{:#016x}",
            vaddr,
            size,
            mem as *mut _ as *const () as usize);*/
        mem.ghc.ghc.set_hr();
        // print!("Waiting...");
        loop 
        {
            print!(".");
            // After a second consider HBA in locked/hung state
            if !mem.ghc.ghc.get_hr()
            {
                // println!("!\nReset successful!");
                println!("OK");
                break;
            }
        }
    }

    fn setup_interrupts(&mut self)
    {
        let interrupt = self.device.get_interrupt_line();
        self.device.set_interrupt_line(11);
        println!("AHCI IRQ: {} => 11 ({})", interrupt, self.device.get_interrupt_line());
    }

    // Change: 10.1.2 "recommends" to always clear the port specific IS register first, then the corresponding HBA IS.IPS.
    /*fn enable_ahci_mode_and_interrupts(&mut self)
    {
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        
        // Enable ahci mode only if legacy mode is supported
        // Otherwise it should be already on and writing is undefined
        if !mem.ghc.cap.get_sam() && !mem.ghc.ghc.get_ae()
        {
            unsafe { mem.ghc.ghc.set_ae(true) }
        }

        // Change: 10.1.2 "recommends" to always clear the port specific IS register first, then the corresponding HBA IS.IPS.
        // I will enable interrupts after the second step.
        // Enable interrupts
        mem.ghc.ghc.set_ie(true);
    }*/

    fn enable_ahci_mode(&mut self)
    {
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        
        // Enable ahci mode only if legacy mode is supported
        // Otherwise it should be already on and writing is undefined
        if !mem.ghc.cap.get_sam() && !mem.ghc.ghc.get_ae()
        {
            unsafe { mem.ghc.ghc.set_ae(true) }
        }
    }

    fn enable_interrupts(&mut self)
    {
        let mem = self.get_hba_mem_mut().expect("Failed to load address");

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
        use crate::arch::x86_64::mm::{
            paging::map,
            physicalmem,
            virtualmem
        };
        let mem = self.get_hba_mem_mut().expect("Failed to load address");
        // let number_of_command_slots = mem.ghc.cap.get_ncs_adjusted();
        // let number_of_ports = mem.ghc.cap.get_np_adjusted();

        // let spin_up = mem.ghc.cap.get_sss();
        // println!("NCS: {}, NP: {}", number_of_command_slots, number_of_ports);
        for i in 0u8..32u8
        {
            if mem.ghc.pi.get(i)
            {
                // bail on atapi? port.cmd.get_atapi
                println!("Handling port {}", i);

                let port = &mut mem.ports[i as usize];
                // println!("- Init");
                let _ = ports::Port::init(port, mem.ghc.cap.get_s64a(), mem.ghc.cap.get_ncs_adjusted(), i);
                // println!("- HBA Clear");
                mem.ghc.is.clear(i);
                // println!("- Interrupts");
                ports::Port::setup_interrupts(port, i);
                // println!("- Start");
                ports::Port::first_start(port);
                // println!("Done");

                /*let port = &mut mem.ports[i as usize];

                port.cmd.get_st();

                let status = port.ssts.get();
                let sig = port.sig.get();
                let clb = ((port.clbu.get() as u64) << 32) | (port.clb.get() as u64);
                let fb = ((port.fbu.get() as u64) << 32) | (port.fb.get() as u64);

                const ADDRESS_MASK: usize = (!0u32) as usize;
                let new_clb = physicalmem::allocate(4096);
                let new_fb = physicalmem::allocate(4096);

                println!("CLB: {:#x} => {:#x}", clb, new_clb);
                println!("FB:  {:#x} => {:#x}", fb, new_fb);
                // let mut fis = fis::RegH2D::default();
                // fis.command.set(0xEC); // ATA_CMD_IDENTIFY
                // fis.pmport_cc.set(1); // pmport 0, c 1

                // All the code currently supports only x86_64, therefor a cfg guard does not make sense
                if !mem.ghc.cap.get_s64a() && (new_clb > 0xff_ff_ff_ffusize || new_fb > 0xff_ff_ff_ffusize)
                {
                    physicalmem::deallocate(new_clb, 4096);
                    physicalmem::deallocate(new_fb, 4096);
                    println!("Could not move memory (new address is 64 bit while the HBA does not support 64 bit).");
                }
                else
                {
                    port.fb.set((new_fb & 0x00_00_00_00_ff_ff_ff_ffusize) as u32);
                    port.fbu.set(((new_fb & 0xff_ff_ff_ff_00_00_00_00usize) >> 32) as u32);

                    port.clb.set((new_clb & 0xff_ff_ff_ff) as u32);
                    port.clbu.set(((new_clb & 0xff_ff_ff_ff_00_00_00_00usize) >> 32) as u32);

                    let vclb = virtualmem::allocate(4096);
                    let vfb = virtualmem::allocate(4096);

                    map::<BasePageSize>(vclb, new_clb, 1, PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITABLE | PageTableEntryFlags::WRITE_THROUGH);
                    map::<BasePageSize>(vfb, new_fb, 1, PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITABLE | PageTableEntryFlags::WRITE_THROUGH);
                }*/
            }
        }
    }

    // OS Dev wiki and 10.1.2 disagree
    // 10.1.2 makes the first step enabling AHCI Mode (which makes sense, as bios os handoff and reset are part of the ahci protocol)
    // but the wiki puts them before ensuring AHCI Mode is enabled
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

        // println!("TODO: Setup interrupts");
        self.setup_interrupts();
        self.enable_ahci_mode();
        self.init_ports();
        self.enable_interrupts();

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
                    "clb", port.clb,
                    "clbu", port.clbu,
                    "fb", port.fb,
                    "fbu", port.fbu,
                    "is", port.is,
                    "ie", port.ie,
                    "cmd", port.cmd,
                    "tfd", port.tfd,
                    "sig", port.sig,
                    "ssts", port.ssts,
                    "sctl", port.sctl,
                    "serr", port.serr
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
    pub ports: [PortRegister]
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
    });
    /*if let Some(it) = unsafe { crate::arch::x86_64::kernel::BOOT_INFO }
    {
        for it in it.memory_map.iter()
        {
            println!("{:?} {:#x}..{:#x}", it.region_type, it.range.start_addr(), it.range.end_addr());
        }
    }*/
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
