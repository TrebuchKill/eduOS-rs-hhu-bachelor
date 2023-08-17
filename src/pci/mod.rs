// NEW

// Warning: This code might not run as intended on non little endian systems

// https://wiki.osdev.org/Pci

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
        unsafe {
            // Disable interrupts
            let interrupt = crate::arch::x86_64::kernel::irq::irq_nested_disable();
            
            // Write the address of the device
            x86::io::outl(CONFIG_ADDRESS, get_address(bus, device, func, offset));

            // Read the value
            let it = x86::io::inl(CONFIG_DATA);

            // Re-enable interrupts when nessecarry
            crate::arch::x86_64::kernel::irq::irq_nested_enable(interrupt);

            // return value
            it
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

macro_rules! define_device_types
{
    ($($name:ident),+) => {
        $(
            #[derive(Clone, Copy, Debug)]
            pub struct $name
            {
                bus: u8,
                device: u8,
                function: u8,
            }

            impl $name
            {
                pub const unsafe fn new_unchecked(bus: u8, device: u8, function: u8) -> Self
                {
                    Self { bus, device, function }
                }

                pub fn new(bus: u8, device: u8, function: u8) -> Option<Self>
                {
                    let dev = unsafe { Self::new_unchecked(bus, device, function) };
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
                    u16::read(self.bus, self.device, self.function, 0x00)
                }

                pub fn get_device_id(&self) -> u16
                {
                    u16::read(self.bus, self.device, self.function, 0x02)
                }

                // TODO: Command Type?
                pub fn get_command(&self) -> u16
                {
                    u16::read(self.bus, self.device, self.function, 0x04)
                }

                pub fn get_status(&self) -> u16
                {
                    u16::read(self.bus, self.device, self.function, 0x06)
                }

                pub fn get_revision_id(&self) -> u8
                {
                    u8::read(self.bus, self.device, self.function, 0x08)
                }

                pub fn get_programming_interface(&self) -> u8
                {
                    u8::read(self.bus, self.device, self.function, 0x09)
                }

                pub fn get_subclass(&self) -> u8
                {
                    u8::read(self.bus, self.device, self.function, 0x0a)
                }

                pub fn get_class(&self) -> u8
                {
                    u8::read(self.bus, self.device, self.function, 0x0b)
                }

                pub fn get_cache_line_size(&self) -> u8
                {
                    u8::read(self.bus, self.device, self.function, 0x0c)
                }

                pub fn get_latency_timer(&self) -> u8
                {
                    u8::read(self.bus, self.device, self.function, 0x0d)
                }

                pub fn get_header_type(&self) -> HeaderType
                {
                    HeaderType(u8::read(self.bus, self.device, self.function, 0x0e))
                }

                // BIST = Built In Self Test
                pub fn get_bist(&self) -> u8
                {
                    u8::read(self.bus, self.device, self.function, 0x0f)
                }
            }

            impl ::core::fmt::Display for $name
            {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
                {
                    write!(f, "({}, {}, {})", self.bus, self.device, self.function)
                }
            }
        )*
    };
}

macro_rules! define_device
{
    ($common:ident, $($name:ident $id:literal),*) => {

        #[derive(Debug, Clone, Copy)]
        #[repr(u8)]
        pub enum KnownHeaderType
        {
            Unknown = 0xff,
            $($name = $id),*
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

        $(
            impl ::core::convert::TryFrom<$common> for $name
            {
                type Error = KnownHeaderType;

                fn try_from(value: $common) -> Result<Self, Self::Error>
                {
                    let typ = value.get_header_type().get_type();
                    if (typ as u8) == $id
                    {
                        Ok(unsafe { Self::new_unchecked(value.bus, value.device, value.function) })
                    }
                    else
                    {
                        Err(typ)
                    }
                }
            }

            impl ::core::convert::From<$name> for $common
            {
                fn from(value: $name) -> Self
                {
                    unsafe { Self::new_unchecked(value.bus, value.device, value.function) }
                }
            }
        )+
    }
}

define_device!(DeviceCommon, DeviceGeneric 0x0, DevicePciBridge 0x1, DeviceCardBridge 0x2);

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
    type Item = DeviceCommon;

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            if self.bdf > 0x00_00_ff_ff
            {
                return None;
            }
            let mf = self.function() > 0;
            if let Some(it) = DeviceCommon::new(self.bus(), self.device(), self.function())
            {
                self.increment(mf || it.get_header_type().is_multifunction());
                return Some(it);
            }
            else
            {
                self.increment(mf);
            }
        }
    }
}

pub fn scan_bus() -> alloc::vec::Vec<DeviceCommon>
{
    PciScanner::new().collect()
}
