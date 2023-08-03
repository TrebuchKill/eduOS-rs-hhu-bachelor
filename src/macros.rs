// EDIT

/// Print formatted text to our console.
///
/// From http://blog.phil-opp.com/rust-os/printing-to-screen.html, but tweaked
/// to work with our APIs.
#[macro_export]
macro_rules! serial_print
{
	($($arg:tt)+) => ($crate::macros::_serial_print(format_args!($($arg)+)));
}

/// Print formatted text to our console, followed by a newline.
///
/// From https://doc.rust-lang.org/nightly/std/macro.println!.html
#[macro_export]
macro_rules! serial_println
{
	() => ($crate::macros::serial_print!("\n"));
	($fmt:expr) => ($crate::macros::serial_print!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => ($crate::macros::serial_print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! vga_print
{
	($($arg:tt)+) => ($crate::macros::_vga_print(format_args!($($arg)+)));
}

#[macro_export]
macro_rules! vga_println
{
	() => ($crate::macros::vga_print!("\n"));
	($fmt:expr) => ($crate::macros::vga_print(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => ($crate::macros::vga_print(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! print
{
	($($arg:tt)+) => ({
		$crate::macros::_serial_print(format_args!($($arg)+));
		$crate::macros::_vga_print(format_args!($($arg)+));
	});
}

#[macro_export]
macro_rules! println
{
	() => ($crate::print!("\n"));
	($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _vga_print(args: core::fmt::Arguments<'_>)
{
	use core::fmt::Write;
	crate::vga::get_buffer().as_mut().unwrap().write_fmt(args).unwrap()
}

#[doc(hidden)]
pub fn _serial_print(args: core::fmt::Arguments<'_>)
{
	use core::fmt::Write;
	crate::console::CONSOLE.lock().write_fmt(args).unwrap();
}

macro_rules! align_down {
	($value:expr, $alignment:expr) => {
		$value & !($alignment - 1)
	};
}

macro_rules! align_up {
	($value:expr, $alignment:expr) => {
		align_down!($value + ($alignment - 1), $alignment)
	};
}
