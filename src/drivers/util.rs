use core::fmt::{
    Result,
    Formatter,
    Display,
    Binary,
    LowerHex,
    UpperHex,
    Debug
};

#[repr(transparent)]
pub struct Register<T>(T);
// Should the getter and setter be unsafe? I mean, they both need to be aligned, and considering how this struct will be used, generally speaking it may not be guranteed.
impl<T> Register<T>
{
    pub const fn new(value: T) -> Self
    {
        Self(value)
    }

    pub fn get(&self) -> T
    {
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    pub fn set(&mut self, value: T)
    {
        unsafe { core::ptr::write_volatile(&mut self.0, value) }
    }
}

// https://stackoverflow.com/questions/71105480/apply-format-arguments-to-struct-members-in-display-implementation

impl<T> Debug for Register<T>
    where T: Debug
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        f.write_str("Register(")?;
        self.0.fmt(f)?;
        f.write_str(")")
    }
}

impl<T> Clone for Register<T>
    where T: Clone
{
    fn clone(&self) -> Self
    {
        Self(self.get().clone())
    }
}

// Copy may be a no-no. Need to check

impl<T> PartialEq for Register<T>
    where T: PartialEq
{
    fn eq(&self, other: &Self) -> bool
    {
        self.get() == other.get()
    }

    fn ne(&self, other: &Self) -> bool
    {
        self.get() != other.get()
    }
}

impl<T> Eq for Register<T>
    where T: Eq
{
}

impl<T> Display for Register<T>
    where T: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        self.0.fmt(f)
    }
}

impl<T> Binary for Register<T>
    where T: Binary
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        self.0.fmt(f)
    }
}

impl<T> LowerHex for Register<T>
    where T: LowerHex
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        self.0.fmt(f)
    }
}

impl<T> UpperHex for Register<T>
    where T: UpperHex
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        self.0.fmt(f)
    }
}

// Are these a good idea? Otherwise I need to change a lot of legacy code.

// What I expect this code to do:
// Implement |= for any R, which can be or'ed together with T ("T | R") returning T
impl<T, R> core::ops::BitOrAssign<R> for Register<T>
    where T: core::ops::BitOr<R, Output = T>
{
    fn bitor_assign(&mut self, rhs: R)
    {
        self.set(self.get() | rhs)
    }
}

// What I expect this code to do:
// Implement &= for any R, which can be and'ed together with T ("T & R") returning T
impl<T, R> core::ops::BitAndAssign<R> for Register<T>
    where T: core::ops::BitAnd<R, Output = T>
{
    fn bitand_assign(&mut self, rhs: R)
    {
        self.set(self.get() & rhs)
    }
}
