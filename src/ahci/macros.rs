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

            $($crate::ahci::macros::define_register!($($rest)+);)?
        }
    };
    // Read Only
    (ro $name_short:ident $name_long:literal $bit:literal $(, $($rest:tt)+)?) => {
        paste::paste!(
            #[doc = $name_long]
            pub fn [<get_ $name_short>](self) -> bool
            {
                self.0 & (1u32 << $bit) != 0
            }
        );

        $($crate::ahci::macros::define_register!($($rest)+);)?
    };
    // Read Write
    (rw $name_short:ident $name_long:literal $bit:literal $(, $($rest:tt)+)?) => {
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

        $($crate::ahci::macros::define_register!($($rest)+);)?
    };
    // Write 1 to clear
    (rwc $name_short:ident $name_long:literal $bit:literal $(, $($rest:tt)+)?) => {
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

        $($crate::ahci::macros::define_register!($($rest)+);)?
    };
    // Write 1 to set
    (rw1 $name_short:ident $name_long:literal $bit:literal $(, $($rest:tt)+)?) => {
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
                // This todo is supposed to give a dead code warning, making me or anyone else validate this assumption!
                todo!("Is this in a register, where only RO, RW1 and RWC are present?");
                self.0 = 1u32 << $bit;
            }
        );

        $($crate::ahci::macros::define_register!($($rest)+);)?
    };
}

pub use define_register;
