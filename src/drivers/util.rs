#[repr(transparent)]
pub struct Register<T>(T);
// Should the getter and setter be unsafe? I mean, they both need to be aligned, and considering how this struct will be used, generally speaking it may not be guranteed.
impl<T> Register<T>
{
    pub fn get(&self) -> T
    {
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    pub fn set(&mut self, value: T)
    {
        unsafe { core::ptr::write_volatile(&mut self.0, value) }
    }
}
