use super::{
    Device,
    DataType,
    super::{
        PCI_LOCK,
        BarValue,
        read,
        write
    }
};

macro_rules! define_bar_accessors {
    ($($getter:ident $address:ident $offset:literal),+) => {
        
        $(pub fn $getter(&self) -> BarValue
        {
            BarValue(u32::read(self.bus, self.device, self.function, $offset))
        }

        // What if IoSpace Address?
        pub fn $address(&self) -> u32
        {
            let lock = PCI_LOCK.lock();
            let (lock, old_value) = read(lock, self.bus, self.device, self.function, $offset);
            let lock = write(lock, self.bus, self.device, self.function, $offset, 0xff_ff_ff_ff);
            let (lock, result) = read(lock, self.bus, self.device, self.function, $offset);
            let _ = write(lock, self.bus, self.device, self.function, $offset, old_value);
            (!(result & 0xff_ff_ff_f0)) + 1
        })+
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Generic
{
    pub(super) bus: u8,
    pub(super) device: u8,
    pub(super) function: u8
}

impl Generic
{
    define_bar_accessors!(
        get_bar_0 get_bar_0_size 0x10,
        get_bar_1 get_bar_1_size 0x14,
        get_bar_2 get_bar_2_size 0x18,
        get_bar_3 get_bar_3_size 0x1c,
        get_bar_4 get_bar_4_size 0x20,
        get_bar_5 get_bar_5_size 0x24
    );
}

impl Device for Generic
{
    fn get_bus(&self) -> u8
    {
        self.bus
    }

    fn get_device(&self) -> u8
    {
        self.device
    }

    fn get_function(&self) -> u8
    {
        self.function
    }
}

impl core::fmt::Display for Generic
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        write!(f, "PCI({}, {}, {})", self.get_bus(), self.get_device(), self.get_function())
    }
}
