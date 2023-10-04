// NEW

use crate::drivers::Register;
use super::Type;

// May I regret not adding packed (here or anywhere else)? On x86 I doubt that, as sizes and alignment of types fit perfectly, but on other platforms?
#[repr(C)]
pub struct RegD2H
{
    pub fis_type: Register<Type>,

    /// pmport 7:4, reserved 3:2, interrupt 1, reserved 0
    pub pmport_cc: Register<u8>,

    /// Status Register
    pub status: Register<u8>,

    /// Error Register
    pub error: Register<u8>,

    /// LBA low Register, 7:0
    pub lba0: Register<u8>,

    /// LBA mid Register, 15:8
    pub lba1: Register<u8>,

    /// LBA high Register, 23:16
    pub lba2: Register<u8>,

    /// Device Register
    pub device: Register<u8>,

    /// LBA Register, 31:24
    pub lba3: Register<u8>,

    /// LBA Register, 39:32
    pub lba4: Register<u8>,

    /// LBA Register, 47:40
    pub lba5: Register<u8>,

    /// Reserved
    reserved_0: Register<u8>,

    /// Count Register: Bits 7:0
    pub countl: Register<u8>,

    /// Count Register: Bits 15:8
    pub counth: Register<u8>,

    /// Reserved
    reserved_1: Register<u8>,

    /// Reserved
    reserved_2: Register<u8>,

    /// Reserved
    reserved_3: Register<[u8; 4]>
}

impl RegD2H
{
    pub const fn default() -> Self
    {
        Self {
            fis_type:   Register::new(Type::REG_D2H),
            pmport_cc:  Register::new(0),
            status:     Register::new(0),
            error:      Register::new(0),
            lba0:       Register::new(0),
            lba1:       Register::new(0),
            lba2:       Register::new(0),
            device:     Register::new(0),
            lba3:       Register::new(0),
            lba4:       Register::new(0),
            lba5:       Register::new(0),
            reserved_0: Register::new(0),
            countl:     Register::new(0),
            counth:     Register::new(0),
            reserved_1: Register::new(0),
            reserved_2: Register::new(0),
            reserved_3: Register::new([0u8; 4]),
        }
    }
}

impl Default for RegD2H
{
    fn default() -> Self
    {
        Self::default()
    }
}