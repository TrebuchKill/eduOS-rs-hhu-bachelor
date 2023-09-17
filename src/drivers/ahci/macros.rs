/*macro_rules! define_getter
{
    ($bit:literal $name_short:ident $name_long:literal) => {
        paste::paste!(
            #[doc = $name_long]
            pub fn [<get_ $name_short>](self) -> bool
            {
                self.0 & (1u32 << $bit) != 0
            }
        );
    };
    ($bit:literal $name_short:ident $name_long:literal $comment:literal) => {

        paste::paste!(
            #[doc = concat!($name_long, "\n\n", $comment)]
            pub fn [<get_ $name_short>](self) -> bool
            {
                self.0 & (1u32 << $bit) != 0
            }
        );
    };
}*/

// 1.5
// TODO: Multi Bit Values
#[macro_export]
macro_rules! define_register
{
    (struct $name:ident $(; $($rest:tt)+)?) => {
        
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(u32);
        impl $name
        {
            pub fn from_raw(value: u32) -> Self
            {
                Self(value)
            }

            pub fn as_raw(self) -> u32
            {
                self.0
            }

            $($crate::drivers::ahci::macros::define_register!($($rest)+);)?
        }
    };
    // Read Only
    (ro $bit:literal $name_short:ident $name_long:literal $(, $($rest:tt)+)?) => {
        paste::paste!(
            #[doc = $name_long]
            pub fn [<get_ $name_short>](self) -> bool
            {
                self.0 & (1u32 << $bit) != 0
            }
        );

        $($crate::drivers::ahci::macros::define_register!($($rest)+);)?
    };
    // Read Write
    (rw $bit:literal $name_short:ident $name_long:literal $(, $($rest:tt)+)?) => {
        paste::paste!(
            #[doc = $name_long]
            pub fn [<get_ $name_short>](self) -> bool
            {
                self.0 & (1u32 << $bit) != 0
            }
        );

        paste::paste!(
            #[doc = $name_long]
            pub fn [<set_ $name_short>](&mut self, value: bool)
            {
                const MASK: u32 = 1u32 << $bit;
                if value
                {
                    self.0 |= MASK;
                }
                else
                {
                    self.0 &= !MASK;
                }
            }
        );

        $($crate::drivers::ahci::macros::define_register!($($rest)+);)?
    };
    // Write 1 to clear
    (rwc $bit:literal $name_short:ident $name_long:literal $(, $($rest:tt)+)?) => {
        paste::paste!(
            #[doc = $name_long]
            pub fn [<get_ $name_short>](self) -> bool
            {
                self.0 & (1u32 << $bit) != 0
            }
        );

        paste::paste!(
            #[doc = $name_long]
            pub fn [<clear_ $name_short>](&mut self)
            {
                self.0 = 1u32 << $bit;
            }
        );

        $($crate::drivers::ahci::macros::define_register!($($rest)+);)?
    };
    // Write 1 to set
    (rw1 $bit:literal $name_short:ident $name_long:literal $(, $($rest:tt)+)?) => {
        paste::paste!(
            #[doc = $name_long]
            pub fn [<get_ $name_short>](self) -> bool
            {
                self.0 & (1u32 << $bit) != 0
            }
        );

        paste::paste!(
            #[doc = $name_long]
            pub fn [<set_ $name_short>](&mut self)
            {
                // This is in a register, where RW are present
                self.0 |= 1u32 << $bit;
            }
        );

        $($crate::drivers::ahci::macros::define_register!($($rest)+);)?
    };
}

pub use define_register;
