// History: I first had two very useless structs, from which one was only used for the functions.
// This here is a complete rewrite of them.

use super::{
    HbaMemory,
    PortRegister,
    fis::{
        CommandListStructure,
        ReceivedFis,
        CommandHeader,
        CommandTable2
    },
    is_ahci_device
};

use crate::{
    LogLevel,
    KernelLogger,
    // LOGGER,
    arch::x86_64::{
        mm::{
            physicalmem,
            virtualmem,
            paging::{
                self,
                PageTableEntryFlags,
                BasePageSize
            }
        },
        kernel::busy_sleep
    },
    synch::spinlock::Spinlock,
    drivers::pci::{
        devices::{Generic as PciGeneric, CommonHeader},
        MemSpaceBarValue
    }
};
use alloc::vec::Vec;
use core::convert::TryFrom;

// TODO: Replace with crate::LOGGER
static LOGGER: KernelLogger = KernelLogger{ log_level: LogLevel::DEBUG };

pub struct AhciDevice2
{
    pub pci_idx: usize,
    pub abar_ptr: &'static mut HbaMemory,
    pub abar_actual_size: usize,
    pub ports: [Option<&'static mut AhciPort2>; 32]
}

impl AhciDevice2
{
    pub unsafe fn init_memory(this: *mut Self, pci_idx: usize, abar_ptr: &'static mut HbaMemory, abar_actual_size: usize) -> &'static mut Self
    {
        use core::ptr::addr_of_mut;
        const PORT_TEMPLATE: Option<&'static mut AhciPort2> = None;

        addr_of_mut!((*this).pci_idx).write_volatile(pci_idx);
        addr_of_mut!((*this).abar_ptr).write_volatile(abar_ptr);
        addr_of_mut!((*this).abar_actual_size).write_volatile(abar_actual_size);
        addr_of_mut!((*this).ports).write_volatile([PORT_TEMPLATE; 32]);

        &mut *this
    }

    pub fn new(pci_idx: usize, device: &PciGeneric, hba_idx: usize) -> Option<Self>
    {
        let mut output = Self::pci_init(pci_idx, device);
        if let Some(ref mut it) = output
        {
            if it.ahci_init(hba_idx)
            {
                return output;
            }
        }
        None
    }

    /// Check if AHCI Device and if so:
    /// - Enable Interrupts (on the PCI level)
    /// - Enable Memory Space Access
    /// - Make Bus Master
    /// - Load the ABAR address and size
    /// - Set Interrupt Line
    fn pci_init(pci_idx: usize, device: &PciGeneric) -> Option<Self>
    {
        const PORT_TEMPLATE: Option<&'static mut AhciPort2> = None;
        if is_ahci_device(device)
        {
            // Load ABAR (BAR[5])
            let paddr = MemSpaceBarValue::try_from(device.get_bar_5()).unwrap();
            let size = device.get_bar_5_size();
            let size_page_aligned = (size + 0x0f_ff) & !0x0f_ffu32;

            let vmem = virtualmem::allocate(size_page_aligned as usize);
            paging::map::<BasePageSize>(
                vmem,
                paddr.address() as usize,
                (size_page_aligned >> 12) as usize,
                PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITABLE | PageTableEntryFlags::WRITE_THROUGH);

            debug_assert_eq!(paging::get_physical_address::<BasePageSize>(vmem), paddr.address() as usize);

            // The bytes for the ports. The ports start at offset 0x01_00 and are each 0x80 in size
            let port_size = size - 0x01_00;
            debug_assert_eq!(port_size % 0x80, 0);
            let port_count = port_size / 0x80;

            // Set the Interrupt Line
            let old_irq = device.get_interrupt_line();
            device.set_interrupt_line(11);
            println!("AHCI IRQ: {} => 11 ({})", old_irq, device.get_interrupt_line());

            // Do the rest
            let mut cmd = device.get_command();
            cmd.set_interrupt_disable(false);
            cmd.set_memory_space(true);
            cmd.set_bus_master(true);
            device.set_command(cmd);

            unsafe
            {   
                Some(Self {

                    pci_idx,
                    abar_ptr: &mut *core::ptr::from_raw_parts_mut(vmem as *mut (), port_count as usize),
                    abar_actual_size: size as usize,
                    ports: [PORT_TEMPLATE; 32]
                })
            }
        }
        else
        {
            None
        }
    }

    fn ahci_init(&mut self, hba_idx: usize) -> bool
    {
        self.bios_os_handoff();
        self.reset();
        
        // ENABLE AHCI MODE
        if !self.abar_ptr.ghc.cap.get_sam() && !self.abar_ptr.ghc.ghc.get_ae()
        {
            unsafe { self.abar_ptr.ghc.ghc.set_ae(true) };
        }
        assert!(self.abar_ptr.ghc.ghc.get_ae(), "AHCI Mode is not enabled.");

        // Enable Ports
        self.init_ports(hba_idx);

        // Enable Interrupts
        self.abar_ptr.ghc.ghc.set_ie(true);

        true
    }

    // 10.6.3
    // Needs Testing
    fn bios_os_handoff(&mut self)
    {
        let mem = &mut self.abar_ptr;
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

                    busy_sleep(30); // Doc says 25, but my code is not "perfect" 1 ms. Hope this will compensate enough.

                    let mut tries = 0u32;
                    while tries < 10 && mem.ghc.bohc.get_bb()
                    {
                        tries += 1;
                        busy_sleep(2000); // Wait the minimum amount of time for the bios to clear this bit, at most 10 times (random choice).
                    }

                    if tries >= 10 && mem.ghc.bohc.get_bb() // after 10 tries of waiting 2 seconds, the bios busy flag is still set. I choose to timeout this operation at this point.
                    {
                        panic!("Bios OS Handoff failed: timeout");
                    }
                    break;
                }
            }
        }
    }

    // 10.4.3
    /// HBA Resets
    /// - GHC.AE
    /// - GHC.IE
    /// - IS Register
    /// - all port register fields except fields intiallized by hardware (HwInit) and PxFB/PxFBU/PxCLB/PxCLBU
    fn reset(&mut self)
    {
        print!("Resetting HBA");
        self.abar_ptr.ghc.ghc.set_hr();
        loop 
        {
            print!(".");
            // TODO: After a second consider HBA in locked/hung state
            if !self.abar_ptr.ghc.ghc.get_hr()
            {
                println!("OK");
                break;
            }
        }
    }

    fn init_ports(&mut self, hba_idx: usize)
    {
        let command_slots_per_port = self.abar_ptr.ghc.cap.get_ncs_adjusted();
        let is_64bit_aware = self.abar_ptr.ghc.cap.get_s64a();

        for i in 0..32u8
        {
            if self.abar_ptr.ghc.pi.get(i)
            {
                self.ports[i as usize] = AhciPort2::new(
                    self,
                    hba_idx,
                    i,
                    command_slots_per_port,
                    is_64bit_aware);
            }
        }
    }
}

impl Drop for AhciDevice2
{
    fn drop(&mut self)
    {
        let ptr = self.abar_ptr as *mut _ as *mut () as usize;
        let size = (self.abar_actual_size + 0x0f_ffusize) & !0x0f_ffusize;

        paging::unmap::<BasePageSize>(ptr, size >> 12);
        virtualmem::deallocate(ptr, size);
    }
}

// Layout for a page:
// Port at offset 0, length 40
// empty space
// FB   at offset 256 (as I do not support frame based switching), length 256
// empty space
// CLB  at offset 1024, length 1024

pub struct AhciPort2
{
    pub hba_idx: usize,
    /// The Index of the Port in the HBA
    pub hba_port_idx: usize,
    // port_mem: &'static mut PortRegister,
    pub clb: &'static mut CommandListStructure,
    pub fb: &'static mut ReceivedFis,
    /// 1 to 32 (inclusive)
    pub cmd_slot_count: u8,
    pub is_64bit_aware: bool,
}

impl AhciPort2
{
    // Not a fan of so many arguments, but my lizzard brain fails to do something more ellegant.
    pub unsafe fn init_memory(
        this: *mut Self,
        hba_idx: usize,
        hba_port_idx: usize,
        // port_mem: &'static mut PortRegister,
        clb: &'static mut CommandListStructure,
        fb: &'static mut ReceivedFis,
        command_slot_count: u8,
        is_64bit_aware: bool) -> &'static mut Self
    {
        use core::ptr::addr_of_mut;

        assert!(command_slot_count > 0 && command_slot_count <= 32, "Only 1 to 32 Command Slots are allowed");

        addr_of_mut!((*this).hba_idx).write_volatile(hba_idx);
        // addr_of_mut!((*this).port_mem).write_volatile(port_mem);
        addr_of_mut!((*this).hba_port_idx).write_volatile(hba_port_idx);
        addr_of_mut!((*this).clb).write_volatile(clb);
        addr_of_mut!((*this).fb).write_volatile(fb);
        addr_of_mut!((*this).cmd_slot_count).write_volatile(command_slot_count);
        addr_of_mut!((*this).is_64bit_aware).write_volatile(is_64bit_aware);

        &mut *this
    }

    pub fn new(
        // The HBA owning this port
        ahci: &mut AhciDevice2,
        // The Index of the HBA in AHCI_DEVICES
        hba_idx: usize,
        // The index of the port inside param ahci
        port_idx: u8,
        // How many command slots?
        command_slot_count: u8,
        // Can we use 64 bit physical address?
        is_64bit_aware: bool) -> Option<&'static mut Self>
    {
        let hba_port_idx = port_idx as usize;
        let (this, clb, fb) = Self::allocate();
        if !Self::stop_impl(&mut ahci.abar_ptr.ports[hba_port_idx])
        {
            Self::deallocate(this);
            return None;
        }

        assert!(!ahci.abar_ptr.ports[hba_port_idx].cmd.get_st());
        assert!(!ahci.abar_ptr.ports[hba_port_idx].cmd.get_cr());
        assert!(!ahci.abar_ptr.ports[hba_port_idx].cmd.get_fre());
        assert!(!ahci.abar_ptr.ports[hba_port_idx].cmd.get_fr());

        // Initialize the values pointed by clb and fb
        unsafe {
            const CL_TEMPLATE: CommandHeader = CommandHeader::new();
            clb.write_volatile([CL_TEMPLATE; 32]);

            const F_TEMPLATE: ReceivedFis = ReceivedFis::default();
            fb.write_volatile(F_TEMPLATE);
        }

        {
            // Setup command list (CLB) and Fis Receive Structure (FB)
            let port = &mut ahci.abar_ptr.ports[hba_port_idx];
            
            let clb64 =
                paging::get_physical_address::<BasePageSize>(clb as usize) as u64;
            if !is_64bit_aware && clb64 > 0xff_ff_ff_ff
            {
                panic!("Hardware does not support 64 bit addressing");
            }
            port.clb.set(clb64 as u32);
            port.clbu.set((clb64 >> 32) as u32);

            let fb64 =
                paging::get_physical_address::<BasePageSize>(fb as usize) as u64;
            if !is_64bit_aware && fb64 > 0xff_ff_ff_ff
            {
                panic!("Hardware does not support 64 bit addressing");
            }
            port.fb.set(fb64 as u32);
            port.fbu.set((fb64 >> 32) as u32);

            port.cmd.set_fre(true);

            // Clear SATA Errors & Diagnostics
            port.serr.set(0xff_ff_ff_ff);

            // Clear all pending interrupt notifications
            port.is.clear_all();
        }

        // Clear the pending interrupt of this port in the HBA
        ahci.abar_ptr.ghc.is.clear(port_idx);

        {
            let port = &mut ahci.abar_ptr.ports[hba_port_idx];

            // Error Interrupts, currently all of them will just panic
            port.ie.set_hbfs(true);
            port.ie.set_hdbs(true);
            port.ie.set_ifs(true);
            port.ie.set_infs(true);
            port.ie.set_ufe(true);
            port.ie.set_ipms(true);

            // Receive Interrupts, a fis was received from the device
            port.ie.set_sdbs(true);
            port.ie.set_dss(true);
            port.ie.set_pss(true);
            port.ie.set_dhrs(true);

            // Start the Port
            port.cmd.set_st(true);
        }

        unsafe {
            Some(
                Self::init_memory(
                    this,
                    hba_idx,
                    hba_port_idx,
                    &mut *clb,
                    &mut *fb,
                    command_slot_count,
                    is_64bit_aware))
        }
    }

    pub fn start(&mut self)
    {
        let mut it = AHCI_DEVICES.lock();
        let hba = &mut it[self.hba_idx];
        Self::stop_impl(&mut hba.abar_ptr.ports[self.hba_port_idx]);
    }

    fn start_impl(port: &mut PortRegister)
    {
        port.cmd.set_fre(true);
        port.cmd.set_st(true);
    }

    pub fn stop(&mut self) -> bool
    {
        let mut it = AHCI_DEVICES.lock();
        let hba = &mut it[self.hba_idx];
        Self::stop_impl(&mut hba.abar_ptr.ports[self.hba_port_idx])
    }

    fn stop_impl(port: &mut PortRegister) -> bool
    {
        // 10.1.2
        if port.cmd.get_st()
        {
            port.cmd.set_st(false);
        }
        // for _ in 0..4 // try 4 times
        // {
        //     if port.cmd.get_cr()
        //     {
        //         busy_sleep(500);
        //     }
        //     else
        //     {
        //         break;
        //     }
        // }
        // if port.cmd.get_cr()
        if !Self::is_commandlist_stopped_impl(port)
        {
            // Maybe add the port number?
            debug!("Timeout waiting for PxCMD.CR");
            return false;
        }

        if port.cmd.get_fre()
        {
            port.cmd.set_fre(false);
        }
        // for _ in 0..4
        // {
        //     if port.cmd.get_fr()
        //     {
        //         busy_sleep(500);
        //     }
        //     else
        //     {
        //         break;
        //     }
        // }
        // if port.cmd.get_fr()
        if !Self::is_fis_receive_disabled_impl(port)
        {
            debug!("Timeout waiting for PxCMD.FR");
            return false;
        }

        true
    }

    /// Spin wait over PxCMD.FRE & PxCMD.CR
    /// 
    /// Before each wait, checks if PxCMD.FR (for FRE) or PxCMD.ST (for CR) is set, bailing with "false" if they are set.
    pub fn is_stopped(&self) -> bool
    {
        let mut it = AHCI_DEVICES.lock();
        let hba = &mut it[self.hba_idx];
        Self::is_stopped_impl(&hba.abar_ptr.ports[self.hba_port_idx])
    }

    fn is_stopped_impl(port: &PortRegister) -> bool
    {
        // FRE must be set before ST, and cleared after ST (and after CR was cleared by the device)
        if !Self::is_commandlist_stopped_impl(port)
        {
            return false;
        }

        Self::is_fis_receive_disabled_impl(port)
    }

    fn is_commandlist_stopped_impl(port: &PortRegister) -> bool
    {
        if port.cmd.get_st()
        {
            return false;
        }
        for _ in 0..60 
        {
            if port.cmd.get_cr()
            {
                busy_sleep(1000);
            }
            else
            {
                break;
            }
        }
        !port.cmd.get_cr()
    }

    fn is_fis_receive_disabled_impl(port: &PortRegister) -> bool
    {
        if port.cmd.get_fre()
        {
            return false;
        }
        for _ in 0..60
        {
            if port.cmd.get_fr()
            {
                busy_sleep(1000);
            }
            else
            {
                break;
            }
        }
        !port.cmd.get_fr()
    }

    /// Allocates and returns Pointers to AhciPort2 (Self), CommandListStructure and ReceivedFis (in this order).
    /// 
    /// All pointers point to uninitiallized memory
    /// 
    /// As all pointers are part of 1 page, it any could be used for a free operation, after moving the pointer to the beginning of the page.
    fn allocate() -> (*mut AhciPort2, *mut CommandListStructure, *mut ReceivedFis)
    {
        let phys = physicalmem::allocate(4096);
        let virt = virtualmem::allocate(4096);
        paging::map::<BasePageSize>(virt, phys, 1, PageTableEntryFlags::WRITABLE | PageTableEntryFlags::CACHE_DISABLE | PageTableEntryFlags::WRITE_THROUGH);

        (virt as *mut AhciPort2, (virt + 2048) as *mut CommandListStructure, (virt + 256) as *mut ReceivedFis)
    }

    /// Requires the AhciPort2 pointer from allocate.
    fn deallocate(this: *mut AhciPort2)
    {
        let phys = paging::get_physical_address::<BasePageSize>(this as usize);
        paging::unmap::<BasePageSize>(this as usize, 1);

        virtualmem::deallocate(this as usize, 4096);
        physicalmem::deallocate(phys, 4096);
    }
}

impl AhciPort2
{
    pub fn write(&mut self, buffer: &[u8])
    {
    }

    pub fn read(&mut self, buffer: &mut [u8])
    {
    }
}

#[doc(hidden)]
pub fn on_interrupt(_num: u8)
{
    super::on_each_device(|i, it| {

		println!("INTERRUPT HBA {}", i);
		println!(
			"- BOH {}, NCS {}, NP  {}",
			it.abar_ptr.ghc.cap2.get_boh(),
			it.abar_ptr.ghc.cap.get_ncs_adjusted(),
			it.abar_ptr.ghc.cap.get_np_adjusted());
		println!("- IS {:032b}, PI  {:032b}", it.abar_ptr.ghc.is, it.abar_ptr.ghc.pi);
		for (j, port) in it.ports.iter().enumerate()
		{
			if let Some(port) = port
			{
				println!("- Port {}", j);
				println!(
					"  - {}, {}, CR {}, FR {}",
					port.hba_idx == i,
					port.hba_port_idx == j,
					it.abar_ptr.ports[port.hba_port_idx].cmd.get_cr(),
					it.abar_ptr.ports[port.hba_port_idx].cmd.get_fr());
				println!("  - CI  {}, CCS {}", it.abar_ptr.ports[port.hba_port_idx].ci.get(), it.abar_ptr.ports[port.hba_port_idx].cmd.get_ccs());
				println!(
					"  - STS {:x}, SSTS {:03x}, SIG {:08x}, SERR {:08x}",
					it.abar_ptr.ports[port.hba_port_idx].tfd.get() & 0xf,
					it.abar_ptr.ports[port.hba_port_idx].ssts.get(),
					it.abar_ptr.ports[port.hba_port_idx].sig.get(),
					it.abar_ptr.ports[port.hba_port_idx].serr.get());
				
			}
		}
	});
}

pub(super) static AHCI_DEVICES: Spinlock<Vec<AhciDevice2>> = Spinlock::new(Vec::new());
// pub(super) static PORTS: Spinlock<Vec<AhciPort2>> = Spinlock::new(Vec::new());
