use crate::drivers::Register;
use super::Type;

#[repr(C)]
pub struct PioSetup
{
    pub fis_type: Register<Type>,
    /// 7:4 Port Multiplier, 3 Reserved, 2 Transfer Direction (1 = device to host), 1 Interrupt Bit, 0 Reserved
    pub pmport_di: Register<u8>,
    /// Status Register
    pub status: Register<u8>,
    /// Error Register
    pub error: Register<u8>,

    /// LBA low register, Bits 7:0
    pub lba0: Register<u8>,
    /// LBA mid register, Bits 15:8
    pub lba1: Register<u8>,
    /// LBA high register, 23:16
    pub lba2: Register<u8>,
    /// Device Register
    pub device: Register<u8>,

    /// LBA Register, Bits 31:24
    pub lba3: Register<u8>,
    /// LBA Register, Bits 39:32
    pub lba4: Register<u8>,
    /// LBA Register, Bits 47:40
    pub lba5: Register<u8>,
    /// Reserved
    reserved_0: Register<u8>,

    /// Count Register, Bits 7:0
    pub countl: Register<u8>,
    /// Count Register, Bits 15:8
    pub counth: Register<u8>,
    /// Reserved
    reserved_1: Register<u8>,
    /// New value of status register
    pub e_status: Register<u8>,

    /// Transfer Count
    pub tc: Register<u16>,
    /// Reserved
    reserved_2: Register<[u8; 2]>
}

impl PioSetup
{
    pub const fn default() -> Self
    {
        Self {
            fis_type: Register::new(Type::PIO_SETUP),
            pmport_di: Register::new(0),
            status: Register::new(0),
            error: Register::new(0),
            lba0: Register::new(0),
            lba1: Register::new(0),
            lba2: Register::new(0),
            device: Register::new(0),
            lba3: Register::new(0),
            lba4: Register::new(0),
            lba5: Register::new(0),
            reserved_0: Register::new(0),
            countl: Register::new(0),
            counth: Register::new(0),
            reserved_1: Register::new(0),
            e_status: Register::new(0),
            tc: Register::new(0),
            reserved_2: Register::new([0; 2])
        }
    }
}

impl Default for PioSetup
{
    fn default() -> Self
    {
        Self::default()
    }
}

impl super::Fis for PioSetup
{
    fn get_type(&self) -> Type
    {
        self.fis_type.get()
    }

    fn copy_into(&self, _dst: &mut [Register<u8>; 64])
    {
        unimplemented!("Send from device only")
    }
}
