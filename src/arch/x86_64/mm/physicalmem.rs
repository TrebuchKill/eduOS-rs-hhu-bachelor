// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use arch::x86_64::mm::paging::{BasePageSize, PageSize};
use arch::x86_64::kernel::get_memory_size;
use collections::Node;
use mm;
use mm::freelist::{FreeList, FreeListEntry};
use mm::POOL;
use scheduler::DisabledPreemption;

static mut PHYSICAL_FREE_LIST: FreeList = FreeList::new();


fn detect_from_limits() -> Result<(), ()> {
	let limit = get_memory_size();

	if limit == 0 {
		return Err(());
	}

	let entry = Node::new(
		FreeListEntry {
			start: mm::kernel_end_address(),
			end: limit
		}
	);
	unsafe { PHYSICAL_FREE_LIST.list.push(entry); }

	Ok(())
}

pub fn init() {
	detect_from_limits().unwrap();
}

pub fn allocate(size: usize) -> usize {
	assert!(size > 0);
	assert!(size % BasePageSize::SIZE == 0, "Size {:#X} is not a multiple of {:#X}", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { PHYSICAL_FREE_LIST.allocate(size) };
	assert!(result.is_ok(), "Could not allocate {:#X} bytes of physical memory", size);
	result.unwrap()
}

pub fn allocate_aligned(size: usize, alignment: usize) -> usize {
	assert!(size > 0);
	assert!(alignment > 0);
	assert!(size % alignment == 0, "Size {:#X} is not a multiple of the given alignment {:#X}", size, alignment);
	assert!(alignment % BasePageSize::SIZE == 0, "Alignment {:#X} is not a multiple of {:#X}", alignment, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe {
		POOL.maintain();
		PHYSICAL_FREE_LIST.allocate_aligned(size, alignment)
	};
	assert!(result.is_ok(), "Could not allocate {:#X} bytes of physical memory aligned to {} bytes", size, alignment);
	result.unwrap()
}

/// This function must only be called from mm::deallocate!
/// Otherwise, it may fail due to an empty node pool (POOL.maintain() is called in virtualmem::deallocate)
pub fn deallocate(physical_address: usize, size: usize) {
	assert!(physical_address >= mm::kernel_end_address(), "Physical address {:#X} is not >= KERNEL_END_ADDRESS", physical_address);
	assert!(size > 0);
	assert!(size % BasePageSize::SIZE == 0, "Size {:#X} is not a multiple of {:#X}", size, BasePageSize::SIZE);

	unsafe { PHYSICAL_FREE_LIST.deallocate(physical_address, size); }
}
