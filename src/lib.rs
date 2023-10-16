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
#![feature(ptr_metadata)] // for src/ahci/mod.rs:115 (Line Number might change without being updated here)
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
use arch::x86_64::kernel::busy_sleep;
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

pub mod drivers;

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

pub fn heap_investigation()
{
	use core::ops::Deref;
	let boot_info = unsafe { crate::arch::x86_64::kernel::BOOT_INFO.unwrap() };
	let tmp = unsafe { HEAP.deref() as *const _ as u64 };
	let start = arch::x86_64::mm::paging::get_physical_address::<arch::x86_64::mm::paging::BasePageSize>(tmp as usize) as u64;
	let end = start + (HEAP_SIZE as u64);
	let mut start_idx = 0usize;
	let mut end_idx = 0usize;
	for (i, it) in boot_info.memory_map.iter().enumerate()
	{
		if it.range.start_addr() <= start && start < it.range.end_addr()
		{
			start_idx = i;
			// println!("HEAP starts in {:?} {:x}..{:x}", it.region_type, it.range.start_addr(), it.range.end_addr());
		}
		if it.range.start_addr() < end && end <= it.range.end_addr()
		{
			end_idx = i;
			// println!("HEAP ends in {:?} {:x}..{:x}", it.region_type, it.range.start_addr(), it.range.end_addr());
		}
	}
	println!("Heap resides in these ranges:");
	for (i, it) in boot_info.memory_map.iter().enumerate()
	{
		if start_idx <= i && i <= end_idx
		{
			println!("- {:?} {:x}..{:x}", it.region_type, it.range.start_addr(), it.range.end_addr());
		}
	}
}

pub extern "C" fn test_c()
{
	test()
}

pub fn test()
{
//	use alloc::vec::Vec;
//	use drivers::pci::devices::{Device, CommonHeader};

//	let mut pci_count = 0;
//	let mut generic_count = 0;
//	let mut card_count = 0;
//	let mut bridge_count = 0;
//	let mut ahci_count = 0;

//	drivers::pci::on_each_device_mut(|_, _| pci_count += 1);
//	drivers::pci::on_each_generic_device_mut(|_, _| generic_count += 1);
//	drivers::pci::on_each_pci_bridge_device_mut(|_, _| bridge_count += 1);
//	drivers::pci::on_each_card_bridge_device_mut(|_, _| card_count += 1);

	/*drivers::pci::on_each_generic_device(|it|
	{
		if drivers::ahci::is_ahci_device(&it)
		{
			println!("Pin:  {}\nLine: {}", it.get_interrupt_pin(), it.get_interrupt_line());
		}
	});*/

//	drivers::ahci::on_each_device_mut(|_, _| ahci_count += 1);
//	drivers::ahci::on_each_device(|i, it| {
//
//		println!("HBA {}", i);
//		println!(
//			"- BOH {}, NCS {}, NP  {}",
//			it.abar_ptr.ghc.cap2.get_boh(),
//			it.abar_ptr.ghc.cap.get_ncs_adjusted(),
//			it.abar_ptr.ghc.cap.get_np_adjusted());
//		println!("- IS {:032b}, PI  {:032b}", it.abar_ptr.ghc.is, it.abar_ptr.ghc.pi);
//		for (j, port) in it.ports.iter().enumerate()
//		{
//			if let Some(port) = port
//			{
//				println!("- Port {}", j);
//				println!(
//					"  - {}, {}, CR {}, FR {}",
//					port.hba_idx == i,
//					port.hba_port_idx == j,
//					it.abar_ptr.ports[port.hba_port_idx].cmd.get_cr(),
//					it.abar_ptr.ports[port.hba_port_idx].cmd.get_fr());
//				println!("  - CI  {}, CCS {}", it.abar_ptr.ports[port.hba_port_idx].ci.get(), it.abar_ptr.ports[port.hba_port_idx].cmd.get_ccs());
//				println!(
//					"  - STS {:x}, SSTS {:03x}, SIG {:08x}, SERR {:08x}",
//					it.abar_ptr.ports[port.hba_port_idx].tfd.get() & 0xf,
//					it.abar_ptr.ports[port.hba_port_idx].ssts.get(),
//					it.abar_ptr.ports[port.hba_port_idx].sig.get(),
//					it.abar_ptr.ports[port.hba_port_idx].serr.get());
//				
//			}
//		}
//	});

//	println!("TEST ({}, {}, {}, {}, {})", pci_count, generic_count, card_count, bridge_count, ahci_count);

	/*let devices = pci::scan_bus();
	println!("Found {} PCI Device(s)", devices.len());
	let ahci_devices = devices.into_iter()
		// .filter(|it| ahci::is_ahci_device(it))
		.filter_map(|it| ahci::AhciDevice::try_new(it))
		.collect::<Vec<_>>();
	println!("Found {} AHCI Device(s)", ahci_devices.len());
	for dev in &ahci_devices
	{
		dev.debug_print();
	}*/

	static mut BUFFER: [u16; 8192] = [0u16; 8192];
	static mut BACKUP: [u16; 8192] = [0u16; 8192];
	let buffer = unsafe { &mut BUFFER };
	let backup = unsafe { &mut BACKUP };
	drivers::ahci::on_each_device_mut(|i, hba| {

		for (j, port) in hba.ports.iter_mut().enumerate()
		{
			if let Some(ref mut port) = port
			{
				*buffer = [0u16; 8192];
				*backup = [0u16; 8192];
				if let Some(bytes_transferred) = port.read_u16(hba.abar_ptr, 0, backup)
				{
					println!("0: HBA: {}, Port: {}, Bytes Read: {}, Last Value: {}, All zero? {}", i, j, bytes_transferred, /*backup[bytes_transferred / 2 - 1]*/"N/A", backup.iter().all(|it| *it == 0));
				}
				if let Some(bytes_transferred) = port.write_u16(hba.abar_ptr, 0, buffer)
				{
					println!("1: HBA: {}, Port: {}, Bytes Written: {}", i, j, bytes_transferred);
				}
				if let Some(bytes_transferred) = port.read_u16(hba.abar_ptr, 0, buffer)
				{
					println!("2: HBA: {}, Port: {}, Bytes Read: {}, Last Value: {}, All zero? {}", i, j, bytes_transferred, /*backup[bytes_transferred / 2 - 1]*/"N/A", buffer.iter().all(|it| *it == 0));
				}
				if let Some(bytes_transferred) = port.write_u16(hba.abar_ptr, 0, backup)
				{
					println!("3: HBA: {}, Port: {}, Bytes Written: {}", i, j, bytes_transferred);
				}
				if let Some(bytes_transferred) = port.read_u16(hba.abar_ptr, 0, buffer)
				{
					println!("4: HBA: {}, Port: {}, Bytes Read: {}, Last Value: {}, All zero? {}", i, j, bytes_transferred, /*backup[bytes_transferred / 2 - 1]*/"N/A", buffer.iter().all(|it| *it == 0));
				}
				println!("All Fine: {}", backup.iter().enumerate().all(|(i, v)| buffer[i] == *v));
			}
		}
		// TODO: REMOVE
		busy_sleep(2000);
	});
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
