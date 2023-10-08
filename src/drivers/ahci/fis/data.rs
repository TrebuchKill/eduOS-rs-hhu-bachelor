// NEW

use crate::drivers::Register;
use super::Type;

#[repr(C)]
pub struct Data
{
    pub fis_type: Register<Type>,
    
    /// pmport 7:4, reserved 3:0
    pub pmport: Register<u8>,

    /// Reserved
    reserved: Register<[u8; 2]>,


    // OsDEV wiki only calls it "DWORD 1 ~ N", without specifying N. And I am split, if this means there must be at least 4 bytes or it can be 0 with this 1 dword being reserved (all zero).
    // Redox uses this field to pad the struct to 256 bytes by making this field 252 bytes long

    // I opted for the redox way, as it makes initializing the data easier

    /// Payload
    pub data: [Register<u8>; 252]
}

impl Data
{
    pub const fn default() -> Self
    {
        const DEFAULT_DATA_VALUE: Register<u8> = Register::new(0);
        Self {

            fis_type: Register::new(Type::DATA),
            pmport: Register::new(0),
            reserved: Register::new([0; 2]),
            data: [DEFAULT_DATA_VALUE; 252]
            // Alternative with inline const
            // See https://github.com/rust-lang/rfcs/pull/2920 https://github.com/rust-lang/rust/issues/76001
            // Currently unstable and behind a feature flag
            // data: [const { Register::new(0) }; 262]
        }
    }
}

impl Default for Data
{
    fn default() -> Self
    {
        Self::default()
    }
}
