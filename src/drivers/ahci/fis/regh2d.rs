// NEW

use crate::drivers::Register;
use super::Type;

// May I regret not adding packed (here or anywhere else)? On x86 I doubt that, as sizes and alignment of types fit perfectly, but on other platforms?
#[repr(C)]
pub struct RegH2D
{
    fis_type: Register<Type>,

    /// pmport 7:4, reserved 3:1, 0: (1 = Command) (0 = Control)
    pmport_cc: Register<u8>,

    /// Command Register
    command: Register<u8>,

    /// Feature Register 7:0
    featurel: Register<u8>,

    /// LBA low Register, 7:0
    lba0: Register<u8>,

    /// LBA mid Register, 15:8
    lba1: Register<u8>,

    /// LBA high Register, 23:16
    lba2: Register<u8>,

    /// Device Register
    device: Register<u8>,

    /// LBA Register, 31:24
    lba3: Register<u8>,

    /// LBA Register, 39:32
    lba4: Register<u8>,

    /// LBA Register, 47:40
    lba5: Register<u8>,

    /// Feature Register, Bits 15:8
    featureh: Register<u8>,

    /// Count Register: Bits 7:0
    countl: Register<u8>,

    /// Count Register: Bits 15:8
    counth: Register<u8>,

    /// Isochronous command completion
    icc: Register<u8>,

    /// Control Register
    control: Register<u8>,

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