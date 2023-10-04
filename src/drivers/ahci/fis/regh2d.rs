// NEW

use crate::drivers::Register;
use super::Type;

// May I regret not adding packed (here or anywhere else)? On x86 I doubt that, as sizes and alignment of types fit perfectly, but on other platforms?
#[repr(C)]
pub struct RegH2D
{
    pub fis_type: Register<Type>,

    /// pmport 7:4, reserved 3:1, 0: (1 = Command) (0 = Control)
    pub pmport_cc: Register<u8>,

    /// Command Register
    pub command: Register<u8>,

    /// Feature Register 7:0
    pub featurel: Register<u8>,

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

    /// Feature Register, Bits 15:8
    pub featureh: Register<u8>,

    /// Count Register: Bits 7:0
    pub countl: Register<u8>,

    /// Count Register: Bits 15:8
    pub counth: Register<u8>,

    /// Isochronous command completion
    pub icc: Register<u8>,

    /// Control Register
    pub control: Register<u8>,

    /// Reserved
    reserved: Register<[u8; 4]>
}

impl RegH2D
{
    pub const fn default() -> Self
    {
        Self {
            fis_type:  Register::new(Type::REG_H2D),
            pmport_cc: Register::new(0),
            command:   Register::new(0),
            featurel:  Register::new(0),
            lba0:      Register::new(0),
            lba1:      Register::new(0),
            lba2:      Register::new(0),
            device:    Register::new(0),
            lba3:      Register::new(0),
            lba4:      Register::new(0),
            lba5:      Register::new(0),
            featureh:  Register::new(0),
            countl:    Register::new(0),
            counth:    Register::new(0),
            icc:       Register::new(0),
            control:   Register::new(0),
            reserved:  Register::new([0u8; 4]),
        }
    }
}

impl Default for RegH2D
{
    fn default() -> Self
    {
        Self::default()
    }
}