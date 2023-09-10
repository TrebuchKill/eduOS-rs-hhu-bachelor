// NEW

mod generic_host_controller;
use generic_host_controller::*;

use crate::{
    pci::{
        devices::{
            Device as PciDevice,
            CommonHeader,
            Generic as PciGeneric
        },
        MemSpaceBarValue,
        BarValue
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
    }
};

use core::convert::{
    TryFrom,
    TryInto
};

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

    pub fn try_new(device: crate::pci::devices::AnyDevice) -> Option<Self>
    {
        use crate::pci::devices::AnyDevice;
        match device
        {
            AnyDevice::Generic(dev) => Self::new(dev),
            _ => None
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

        let addr = MemSpaceBarValue::try_from(self.device.get_bar_5()).unwrap();
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
        self.abar_actual_size = size as usize;
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
        println!("{}: {:x}", "ghc", it.ghc.ghc);
        println!("{}: {:x}", "is", it.ghc.is);
        println!("{}: {:032b}", "pi", it.ghc.pi);
        println!("{}: {:x}", "vs", it.ghc.vs);
        println!("{}: {:x}", "ccc_ctl", it.ghc.ccc_ctl);
        println!("{}: {:x}", "ccc_ports", it.ghc.ccc_ports);
        println!("{}: {:x}", "em_loc", it.ghc.em_loc);
        println!("{}: {:x}", "em_ctl", it.ghc.em_ctl);
        println!("{}: {:x}", "cap2", it.ghc.cap2);
        println!("{}: {:x}", "bohc", it.ghc.bohc);
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

// Alignment is 4
// this fits into the 0x80 spacing between ports
// first port (0) starts at 0x100 (relative to the beginning of the HBA Memory Registers)
// second port (1) starts at 0x180
// the last potential port (31) starts at 0x1080
// if port 30 and therefor port 31 are present, as they start at 0x1000, more than one page will be required for mapping
// AHCI Spec 3.3
#[repr(C)]
pub struct PortRegister
{
    /// Lower 32-bit Command List Base address
    clb: u32,
    /// Higher 32-bit Command List Base address
    clbu: u32,
    /// lower 32-bit Fis Base address
    fb: u32,
    /// higher 32-bit Fis Base address
    fbu: u32,
    /// Interrupts Status
    is: u32,
    /// Interrupt Enable, not Internet Explorer
    ie: u32,
    /// Command and Status
    cmd: u32,
    /// reserved, like always, should be 0
    _reserved: u32,
    /// Task File Data
    tfd: u32,
    /// Signature
    sig: u32,
    /// Serial ata STatuS
    ssts: u32,
    /// Serial ata ConTroL
    sctl: u32,
    /// Serial ata ERRor
    serr: u32
}

// because I had no luck with #[repr(C, align(0x80))]
pub struct AlignedPortRegisters
{
    pub value: PortRegister,
    _padding: [u8; 0x4c]
}

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
