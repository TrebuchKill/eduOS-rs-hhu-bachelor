// EDIT

#![feature(lang_items)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(panic_info_message)]
#![feature(linked_list_cursors)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(specialization)]
#![no_std]

extern crate alloc;
extern crate spin;
#[cfg(target_arch = "x86_64")]
extern crate x86;
#[macro_use]
extern crate bitflags;
extern crate goblin;
extern crate num_traits;

// These need to be visible to the linker, so we need to export them.
use crate::consts::HEAP_SIZE;
#[cfg(target_arch = "x86_64")]
use arch::processor::*;
use core::panic::PanicInfo;
pub use logging::*;
use simple_chunk_allocator::{heap, heap_bitmap, GlobalChunkAllocator, PageAligned};

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod arch;
pub mod collections;
pub mod console;
pub mod vga;
pub mod consts;
pub mod errno;
pub mod fs;
pub mod mm;
pub mod scheduler;
pub mod synch;
pub mod syscall;

pub mod ahci;
pub mod pci;

// Using the Simple Chunk Allocator for heap managment of the kernel
// see
const CHUNK_SIZE: usize = 256;
const CHUNK_AMOUNT: usize = HEAP_SIZE / CHUNK_SIZE;

static mut HEAP: PageAligned<[u8; HEAP_SIZE]> =
	heap!(chunks = CHUNK_AMOUNT, chunksize = CHUNK_SIZE);
static mut HEAP_BITMAP: PageAligned<[u8; CHUNK_AMOUNT / 8]> = heap_bitmap!(chunks = CHUNK_AMOUNT);

#[global_allocator]
static ALLOCATOR: GlobalChunkAllocator =
	unsafe { GlobalChunkAllocator::new(HEAP.deref_mut_const(), HEAP_BITMAP.deref_mut_const()) };

pub extern "C" fn test_c()
{
	test()
}

pub fn test()
{
	use core::convert::TryFrom;

	let devices = pci::scan_bus();
	println!("Found {} PCI Devices", devices.len());
	for device in devices
	{
		print!("Type: {}", device.get_header_type().get_type());
		if device.get_class() == 0x01
		{
			if device.get_subclass() == 0x06
			{
				if device.get_programming_interface() == 0x01
				{
					print!(" AHCI Device {}!", device);
				}
				else
				{
					print!(" SATA Controller {}!", device);
				}
			}
			else
			{
				print!(" Mass Storage {}!", device);
			}
		}
		
		print!("({}, {}, {})",
			if let Ok(_) = pci::DeviceGeneric::try_from(device)
			{ "+" } else { "-" },
			if let Ok(_) = pci::DevicePciBridge::try_from(device)
			{ "+" } else { "-" },
			if let Ok(_) = pci::DeviceCardBridge::try_from(device)
			{ "+" } else { "-" }
		);
		println!();
	}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
	let tid = scheduler::get_current_taskid();

	print!("[!!!PANIC from task {}!!!] ", tid);

	if let Some(location) = info.location() {
		print!("{}:{}: ", location.file(), location.line());
	}

	if let Some(message) = info.message() {
		print!("{}", message);
	}

	print!("\n");

	loop {
		halt();
	}
}
