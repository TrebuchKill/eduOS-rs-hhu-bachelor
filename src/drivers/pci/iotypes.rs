// NEW
use super::{
    PCI_LOCK,
    read,
    write
};

pub trait DataType: Copy
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self;
    fn write(self, bus: u8, device: u8, func: u8, offset: u8);
}

impl DataType for u8
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        // let read = u32::read(bus, device, func, offset);
        let (_, read) = read(PCI_LOCK.lock(), bus, device, func, offset);

        match offset & 0x03
        {
            0 => (read & 0x00_00_00_ff) as u8,
            1 => ((read & 0x00_00ff_00) >> 8) as u8,
            2 => ((read & 0x00_ff_00_00) >> 16) as u8,
            3 => ((read & 0xff_00_00_00) >> 24) as u8,
            _ => panic!("Illegal Value")
        }
    }

    fn write(self, bus: u8, device: u8, func: u8, offset: u8)
    {
        let lock = PCI_LOCK.lock();
        let (lock, value) = read(lock, bus, device, func, offset & 0xfc);
        let new_value = match offset & 0x3
        {
            0 => (value & 0xff_ff_ff_00) | (self as u32),
            1 => (value & 0xff_ff_00_ff) | ((self as u32) << 8),
            2 => (value & 0xff_00_ff_ff) | ((self as u32) << 16),
            3 => (value & 0x00_ff_ff_ff) | ((self as u32) << 24),
            _ => unreachable!()
        };
        let _ = write(lock, bus, device, func, offset & 0xfc, new_value);
    }
}

impl DataType for u16
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        //let read = u32::read(bus, device, func, offset);
        let (_, read) = read(PCI_LOCK.lock(), bus, device, func, offset);

        match offset & 0x03
        {
            0 => (read & 0x00_00_ff_ff) as u16,
            2 => ((read & 0xff_ff_00_00) >> 16) as u16,
            1 | 3 => panic!("Offset must be 16 bits aligned"),
            _ => panic!("Illegal Value")
        }
    }

    fn write(self, bus: u8, device: u8, func: u8, offset: u8)
    {
        match offset & 0x03
        {
            0 | 2 => (),
            _ => panic!("u16 access must be 2 bytes aligned")
        }

        let lock = PCI_LOCK.lock();
        let (lock, value) =
            read(lock, bus, device, func, offset & 0xfc);
        let new_value = match offset & 0x03
        {
            0 => (value & 0xff_ff_00_00) | (self as u32),
            2 => (value & 0x00_00_ff_ff) | ((self as u32) << 16),
            _ => unreachable!()
        };
        let _ = write(lock, bus, device, func, offset & 0xfc, new_value);
    }
}

impl DataType for u32
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        let (_, read) = read(PCI_LOCK.lock(), bus, device, func, offset);
        read
    }

    fn write(self, bus: u8, device: u8, func: u8, offset: u8)
    {
        let _ = write(PCI_LOCK.lock(), bus, device, func, offset, self);
    }
}

macro_rules! define_bit_bool_access
{
    ($getter:ident $setter:ident $mask:literal) => {
        
        pub fn $setter(&mut self, set: bool)
        {
            let tmp = self.0 & (!$mask);
            self.0 = if set
            {
                tmp | $mask
            }
            else
            {
                tmp
            };
        }

        define_bit_bool_access!($getter $mask);
    };
    ($getter:ident $mask:literal) => {

        pub fn $getter(&self) -> bool
        {
            self.0 & $mask == $mask
        }
    };
}

// bitflags! seems to not be what I want (some readonly bits, some read write bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command(u16);

impl Command
{
    define_bit_bool_access!(get_io_space set_io_space 0x00_01);
    define_bit_bool_access!(get_memory_space set_memory_space 0x00_02);
    define_bit_bool_access!(get_bus_master set_bus_master 0x00_04);
    define_bit_bool_access!(get_special_cycles 0x00_08);
    define_bit_bool_access!(get_memory_write_and_invalidate_enable 0x00_10);
    define_bit_bool_access!(get_vga_palette_snoop 0x00_20);
    define_bit_bool_access!(get_parity_error_response set_parity_error_response 0x00_40);
    define_bit_bool_access!(get_serr_enable set_serr_enable 0x01_00);
    define_bit_bool_access!(get_fast_b2b_enable 0x02_00);
    define_bit_bool_access!(get_interrupt_disable set_interrupt_disable 0x04_00);
}

impl core::fmt::Display for Command
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        let mut any = false;
        write!(f, "Command: ")?;
        if self.get_io_space()
        {
            write!(f, "IO Space")?;
            any = true;
        }
        if self.get_memory_space()
        {
            write!(f, "{}Memory Space", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_bus_master()
        {
            write!(f, "{}Bus Master", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_special_cycles()
        {
            write!(f, "{}Special Cycles", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_memory_write_and_invalidate_enable()
        {
            write!(f, "{}Memory Write and Invalidate enable", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_vga_palette_snoop()
        {
            write!(f, "{}VGA Palette Snoop", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_parity_error_response()
        {
            write!(f, "{}Parity Error Response", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_serr_enable()
        {
            write!(f, "{}SERR# enable", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_fast_b2b_enable()
        {
            write!(f, "{}Fast Back-to-Back enable", if any { ", " } else { "" })?;
            any = true;
        }
        if self.get_interrupt_disable()
        {
            write!(f, "{}Interupt Disabled", if any { ", " } else { "" })?;
            any = true;
        }
        if !any
        {
            write!(f, "None")?;
        }
        Ok(())
    }
}

impl DataType for Command
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        Self(u16::read(bus, device, func, offset))
    }

    fn write(self, bus: u8, device: u8, func: u8, offset: u8)
    {
        self.0.write(bus, device, func, offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status(u16);

impl Status
{
    pub const DEFAULT_OFFSET: u16 = 0x04;

    pub fn get_interrupt_status(&self) -> bool
    {
        self.0 & 0x00_08 == 0x00_08
    }

    pub fn get_capabilities_list(&self) -> bool
    {
        self.0 & 0x00_10 == 0x00_10
    }

    pub fn get_66mhz_capable(&self) -> bool
    {
        self.0 & 0x00_20 == 0x00_20
    }

    // back-to-back
    pub fn get_fast_b2b_capable(&self) -> bool
    {
        self.0 & 0x00_80 == 0x00_80
    }

    // todo
}

impl DataType for Status
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        Self(u16::read(bus, device, func, offset))
    }

    fn write(self, bus: u8, device: u8, func: u8, offset: u8)
    {
        self.0.write(bus, device, func, offset)
    }
}
