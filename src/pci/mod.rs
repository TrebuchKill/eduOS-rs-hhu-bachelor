// NEW

// Warning: This code might not run as intended on non little endian systems

// https://wiki.osdev.org/Pci

use crate::synch::spinlock::{SpinlockIrqSave, SpinlockIrqSaveGuard};

const CONFIG_ADDRESS: u16 = 0x0c_f8;
const CONFIG_DATA:    u16 = 0x0c_fc;

fn get_address(bus: u8, device: u8, func: u8, offset: u8) -> u32
{
    const ENABLE_BIT: u32 = 0x80_00_00_00;
    // RESERVED 0x7f_00_00_00;
    // const BUS:       u32 = 0x00_ff_00_00;
    // const DEVICE:    u32 = 0x00_00_f8_00;
    // const FUNCTION:  u32 = 0x00_00_07_00;
    const OFFSET:    u32 = 0x00_00_00_fc;

    // assert_eq!(device & 0x1f, device); // Are only the 5 least significant bits set?
    // assert_eq!(func   & 0x07, func);   // Are only the 3 least significant bits set?

    let bus = bus as u32;
    let dev = device as u32;
    let fun = func as u32;
    let off = offset as u32;

    ENABLE_BIT | (bus << 16) | (dev << 11) | (fun << 8) | (off & OFFSET)
}

static PCI_LOCK: SpinlockIrqSave<()> = SpinlockIrqSave::new(());

fn read<'a>(guard: SpinlockIrqSaveGuard<'a, ()>, bus: u8, device: u8, func: u8, offset: u8)
    -> (SpinlockIrqSaveGuard<'a, ()>, u32)
{
    unsafe
    {
        // Write the address of the device
        x86::io::outl(CONFIG_ADDRESS, get_address(bus, device, func, offset));

        // Read the value
        let it = x86::io::inl(CONFIG_DATA);

        (guard, it)
    }
}

fn write<'a>(guard: SpinlockIrqSaveGuard<'a, ()>, bus: u8, device: u8, func: u8, offset: u8, value: u32)
    -> SpinlockIrqSaveGuard<'a, ()>
{
    unsafe
    {
        x86::io::outl(CONFIG_ADDRESS, get_address(bus, device, func, offset));
        x86::io::outl(CONFIG_DATA, value);
    }
    guard
}

mod iotypes;
use iotypes::DataType;

pub mod devices;

#[derive(Debug)]
pub struct BarValue(u32);
#[derive(Debug)]
pub struct MemSpaceBarValue(u32);
#[derive(Debug)]
pub struct IoSpaceBarValue(u32);

#[derive(Clone, Copy, Debug)]
pub enum MemSpaceType
{
    Bits32    = 0x0,
    Reserved  = 0x1,
    Bits64    = 0x2,
    Undefined = 0x3
}

impl MemSpaceBarValue
{
    pub fn typ(&self) -> MemSpaceType
    {
        use MemSpaceType::*;
        match (self.0 & 6) >> 1
        {
            0 => Bits32,
            2 => Bits64,
            1 => Reserved,
            3 => Undefined,
            _ => unreachable!()
        }
    }

    pub fn is_prefetchable(&self) -> bool
    {
        self.0 & 8 == 8
    }

    // Does not handle Bits64
    pub fn address(&self) -> u32
    {
        self.0 & 0xff_ff_ff_f0
    }
}

impl IoSpaceBarValue
{
    pub fn address(&self) -> u32
    {
        self.0 & 0xff_ff_ff_fc
    }
}

impl core::convert::TryFrom<BarValue> for MemSpaceBarValue
{
    type Error = &'static str;

    fn try_from(value: BarValue) -> Result<Self, Self::Error>
    {
        if value.0 & 1 != 0
        {
            Err("BarValue has not the right format")
        }
        else
        {
            Ok(MemSpaceBarValue(value.0))
        }
    }
}

impl core::convert::TryFrom<BarValue> for IoSpaceBarValue
{
    type Error = &'static str;

    fn try_from(value: BarValue) -> Result<Self, Self::Error>
    {
        if value.0 & 1 != 1
        {
            Err("BarValue has not the right format")
        }
        else
        {
            Ok(IoSpaceBarValue(value.0))
        }
    }
}

#[derive(Debug)]
pub struct PciScanner
{
    // Not what I had in mind originally, but it worked out at the end
    // If any of the high value 16 bits are set, we are done
    // the high byte of the low 16 bits are the Bus
    // the highest 5 bits of the lowest byte are the Device number
    // the remaining lowest 3 bits are the Function
    bdf: u32
}

impl PciScanner
{
    pub const fn new() -> Self
    {
        PciScanner { bdf: 0 }
    }

    fn increment(&mut self, multi_function: bool) // -> bool
    {
        if multi_function
        {
            self.bdf += 1;
        }
        else
        {
            self.bdf = (self.bdf + 0x08) & 0xff_ff_ff_f8;
        }
    }

    pub fn bus(&self) -> u8
    {
        ((self.bdf & 0x00_00_ff_00) >> 8) as u8
    }

    pub fn device(&self) -> u8
    {
        ((self.bdf & 0x00_00_00_f8) >> 3) as u8
    }

    pub fn function(&self) -> u8
    {
        (self.bdf & 0x7) as u8
    }
}

impl Iterator for PciScanner
{
    type Item = devices::AnyDevice;

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            if self.bdf > 0x00_00_ff_ff
            {
                return None;
            }
            let mf = self.function() > 0;
            if let Some(it) = devices::AnyDevice::new(self.bus(), self.device(), self.function())
            {
                self.increment(mf || it.as_common_header().get_header_type().is_multifunction());
                return Some(it);
            }
            else
            {
                self.increment(mf);
            }
        }
    }
}

pub fn scan_bus() -> alloc::vec::Vec<devices::AnyDevice>
{
    PciScanner::new().collect()
}
