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
        let _ = bus;
        let _ = device;
        let _ = func;
        let _ = offset;
        todo!();
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
        let _ = bus;
        let _ = device;
        let _ = func;
        let _ = offset;
        todo!()
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
