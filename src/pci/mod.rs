// NEW

// Warning: This code might not run as intended on non little endian systems

// https://wiki.osdev.org/Pci

const ConfigAddress: u16 = 0x0c_f8;
const ConfigData:    u16 = 0x0c_fc;

fn get_address(bus: u8, device: u8, func: u8, offset: u8) -> u32
{
    const ENABLE_BIT: u32 = 0x80_00_00_00;
    // RESERVED 0x7f_00_00_00;
    // const BUS:       u32 = 0x00_ff_00_00;
    // const DEVICE:    u32 = 0x00_00_f8_00;
    // const FUNCTION:  u32 = 0x00_00_07_00;
    const OFFSET:    u32 = 0x00_00_00_fc;

    assert_eq!(device & 0x1f, device); // Are only the 5 least significant bits set?
    assert_eq!(func   & 0x07, func);   // Are only the 3 least significant bits set?

    let bus = bus as u32;
    let dev = device as u32;
    let fun = func as u32;
    let off = offset as u32;

    ENABLE_BIT | (bus << 16) | (dev << 11) | (fun << 8) | (off & OFFSET)
}

trait DataType: Copy
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self;
    fn write(self, bus: u8, device: u8, func: u8, offset: u8);
}

impl DataType for u8
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        let read = u32::read(bus, device, func, offset);

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
        todo!();
    }
}

impl DataType for u16
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        let read = u32::read(bus, device, func, offset);

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
        todo!()
    }
}

impl DataType for u32
{
    fn read(bus: u8, device: u8, func: u8, offset: u8) -> Self
    {
        unsafe {
            // Disable interrupts
            let interrupt = crate::arch::x86_64::kernel::irq::irq_nested_disable();
            
            // Write the address of the device
            x86::io::outl(ConfigAddress, get_address(bus, device, func, offset));

            // Read the value
            let it = x86::io::inl(ConfigData);

            // Re-enable interrupts when nessecarry
            crate::arch::x86_64::kernel::irq::irq_nested_enable(interrupt);

            // return value
            it
        }
    }

    fn write(self, bus: u8, device: u8, func: u8, offset: u8)
    {
        todo!()
    }
}

macro_rules! define_device_types
{
    ($($name:ident),+) => {
        $(
            #[derive(Clone, Copy, Debug)]
            pub struct $name
            {
                bus: u8,
                device: u8
            }

            impl $name
            {
                pub const unsafe fn new_unchecked(bus: u8, device: u8) -> Self
                {
                    Self { bus, device }
                }

                pub fn new(bus: u8, device: u8) -> Option<Self>
                {
                    let dev = unsafe { Self::new_unchecked(bus, device) };
                    if dev.get_vendor_id() != 0xff_ffu16
                    {
                        Some(dev)
                    }
                    else
                    {
                        None
                    }
                }

                pub fn get_vendor_id(&self) -> u16
                {
                    u16::read(self.bus, self.device, 0, 0x00)
                }

                pub fn get_device_id(&self) -> u16
                {
                    u16::read(self.bus, self.device, 0, 0x02)
                }

                // TODO: Command Type?
                pub fn get_command(&self) -> u16
                {
                    u16::read(self.bus, self.device, 0, 0x04)
                }

                pub fn get_status(&self) -> u16
                {
                    u16::read(self.bus, self.device, 0, 0x06)
                }

                pub fn get_revision_id(&self) -> u8
                {
                    u8::read(self.bus, self.device, 0, 0x08)
                }

                pub fn get_programming_interface(&self) -> u8
                {
                    u8::read(self.bus, self.device, 0, 0x09)
                }

                pub fn get_subclass(&self) -> u8
                {
                    u8::read(self.bus, self.device, 0, 0x0a)
                }

                pub fn get_class(&self) -> u8
                {
                    u8::read(self.bus, self.device, 0, 0x0b)
                }

                pub fn get_cache_line_size(&self) -> u8
                {
                    u8::read(self.bus, self.device, 0, 0x0c)
                }

                pub fn get_latency_timer(&self) -> u8
                {
                    u8::read(self.bus, self.device, 0, 0x0d)
                }

                pub fn get_header_type(&self) -> HeaderType
                {
                    HeaderType(u8::read(self.bus, self.device, 0, 0x0e))
                }

                // BIST = Built In Self Test
                pub fn get_bist(&self) -> u8
                {
                    u8::read(self.bus, self.device, 0, 0x0f)
                }
            }
        )*
    };
}

macro_rules! define_device
{
    ($common:ident, $($name:ident $id:literal),*) => {

        #[derive(Debug)]
        pub enum KnownHeaderType
        {
            Unknown,
            $($name),*
        }

        impl core::fmt::Display for KnownHeaderType
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
            {
                use KnownHeaderType::*;
                match *self
                {
                    Unknown => write!(f, "Unknown"),
                    $($name => write!(f, "{}", stringify!($name))),*
                }
            }
        }

        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct HeaderType(u8);
        
        impl HeaderType
        {
            pub fn is_multifunction(self) -> bool
            {
                self.0 & 0x80 == 0x80
            }
        
            pub fn get_type(self) -> KnownHeaderType
            {
                use KnownHeaderType::*;
                match self.0 & 0x7f
                {
                    $($id => $name,)*
                    _ => Unknown
                }
            }
        }

        define_device_types!($common, $($name),*);
    }
}

define_device!(DeviceCommon, DeviceGeneric 0x0, DevicePciBridge 0x1, DeviceCardBridge 0x2);

impl core::convert::TryInto<DeviceGeneric> for DeviceCommon
{
    type Error = KnownHeaderType;

    fn try_into(self) -> Result<DeviceGeneric, Self::Error>
    {
        // Header Type 0x0
        todo!()
    }
}

impl core::convert::TryInto<DevicePciBridge> for DeviceCommon
{
    type Error = KnownHeaderType;

    fn try_into(self) -> Result<DevicePciBridge, Self::Error>
    {
        // Header Type 0x1
        todo!()
    }
}

impl core::convert::TryInto<DeviceCardBridge> for DeviceCommon
{
    type Error = KnownHeaderType;

    fn try_into(self) -> Result<DeviceCardBridge, Self::Error>
    {
        // Header Type 0x2
        todo!()
    }
}

/*impl DeviceCommon
{
    pub const unsafe fn new_unchecked(bus: u8, device: u8) -> Self
    {
        Self{ bus, device }
    }

    pub fn new(bus: u8, device: u8) -> Self
    {
        todo!()
    }
}*/

pub fn scan_bus() -> alloc::vec::Vec<DeviceCommon>
{
    let mut devices = alloc::vec::Vec::new();
    // 8 Bits
    for bus in 0..=0xffu8
    {
        // 5 Bits
        for dev in 0..=0x1fu8
        {
            if let Some(it) = DeviceCommon::new(bus, dev)
            {
                devices.push(it)
            }
        }
    }
    devices
}
