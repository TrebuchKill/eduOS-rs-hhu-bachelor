// EDIT

#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate eduos_rs;
extern crate alloc;
#[cfg(target_arch = "x86_64")]
extern crate x86;

use alloc::string::String;
use eduos_rs::arch;
use eduos_rs::arch::load_application;
use eduos_rs::fs;
use eduos_rs::mm;
use eduos_rs::scheduler;
use eduos_rs::scheduler::task::{ LOW_PRIORITY, NORMAL_PRIORITY };
use eduos_rs::{LogLevel, LOGGER};
use eduos_rs::drivers;

extern "C" fn create_user_foo() {
	let path = String::from("/bin/demo");

	info!("Hello from loader");

	// load application
	if load_application(&path).is_err() {
		error!("Unable to load elf64 binary {}", path)
	}
}

extern "C" fn foo() {
	let tid = scheduler::get_current_taskid();

	println!("hello from task {}", tid);
}

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> ! {
	arch::init();
	mm::init();
	scheduler::init();
	fs::init();
	// My busy_wait function requires the pit, which requires interrupts.
	// this is why I enable them
	// I need to look into it, but test seem to be fine.
	arch::irq::irq_enable();
	drivers::init();
	arch::irq::irq_disable();

	println!("Hello from eduOS-rs!");

	info!("Print file system:");
	fs::lsdir().unwrap();

	for _i in 0..2 {
		scheduler::spawn(foo, NORMAL_PRIORITY).unwrap();
	}
	scheduler::spawn(create_user_foo, NORMAL_PRIORITY).unwrap();
	scheduler::spawn(eduos_rs::test_c, LOW_PRIORITY).unwrap();

	// enable interrupts => enable preemptive multitasking
	arch::irq::irq_enable();

	scheduler::reschedule();

	println!("Shutdown system!");

	// Keep the QEMU window alive
	// TODO: Remove
	loop {
		unsafe { x86::halt() };
	}
	// shutdown system
	arch::processor::shutdown();
}
