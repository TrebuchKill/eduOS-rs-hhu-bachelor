// NEW

// Why not enum?
// Values 0 and 4 to 15 are reserved
// A future device may implement a newer version of the standard with a currently undefined speed.
// With a rust enum, this would result in undefined behavior.
//   Assuming I do not define 13 different "reserved" values in the enum.
// With this struct, it will just fail to compare with any of these values.
/// Any 4-bit not defined value in InterfaceSpeed has to be considered "reserved"
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterfaceSpeed(pub(super) u8);
impl InterfaceSpeed
{
    /// 1.5 Gbps
    pub const GEN1: Self = Self(1);

    /// 3 Gbps
    pub const GEN2: Self = Self(2);

    /// 6 Gbps
    pub const GEN3: Self = Self(3);
}

impl core::fmt::Binary for InterfaceSpeed
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        if f.alternate()
        {
            write!(f, "{:#04b}", self.0)
        }
        else
        {
            write!(f, "{:04b}", self.0)
        }
    }
}

impl core::fmt::Display for InterfaceSpeed
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        match *self
        {
            Self::GEN1 => write!(f, "Gen 1 (1.5 Gbps)"),
            Self::GEN2 => write!(f, "Gen 2 (3 Gbps)"),
            Self::GEN3 => write!(f, "Gen 3 (6 Gbps)"),
            Self(it) => write!(f, "Reserved/Unknown (Value: {:b})", it)
        }
    }
}
