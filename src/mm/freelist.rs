// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use alloc::rc::Rc;
use core::cell::RefCell;
use collections::{DoublyLinkedList, Node};
use mm;
use logging::*;

pub struct FreeListEntry {
	pub start: usize,
	pub end: usize,
}

pub struct FreeList {
	pub list: DoublyLinkedList<FreeListEntry>,
}

impl FreeList {
	pub const fn new() -> Self {
		Self { list: DoublyLinkedList::new() }
	}

	pub fn allocate(&mut self, size: usize) -> Result<usize, ()> {
		debug!("Allocating {} bytes from Free List {:#X}", size, self as *const Self as usize);

		// Find a region in the Free List that has at least the requested size.
		for node in self.list.iter() {
			let (region_start, region_size) = {
				let borrowed = node.borrow();
				(borrowed.value.start, borrowed.value.end - borrowed.value.start)
			};

			if region_size > size {
				// We have found a region that is larger than the requested size.
				// Return the address to the beginning of that region and shrink the region by that size.
				node.borrow_mut().value.start += size;
				return Ok(region_start);
			} else if region_size == size {
				// We have found a region that has exactly the requested size.
				// Return the address to the beginning of that region and move the node into the pool for deletion or reuse.
				self.list.remove(node.clone());
				unsafe { mm::POOL.list.push(node); }
				return Ok(region_start);
			}
		}

		Err(())
	}

	#[inline]
	fn allocate_address_for_node(&mut self, address: usize, end: usize, node: Rc<RefCell<Node<FreeListEntry>>>) -> bool {
		let (region_start, region_end) = {
			let borrowed = node.borrow();
			(borrowed.value.start, borrowed.value.end)
		};

		// There are 4 possible cases of finding the free space we want to reserve.
		if region_start == address && region_end == end {
			// We found free space that has exactly the address and size of the block we want to allocate.
			// Remove it.
			self.list.remove(node.clone());
			unsafe { mm::POOL.list.push(node); }
			return true;
		} else if region_start < address && region_end == end {
			// We found free space in which the block we want to allocate lies right-aligned.
			// Resize the free space to end at our block.
			node.borrow_mut().value.end = address;
			return true;
		} else if region_start == address && region_end > end {
			// We found free space in which the block we want to allocate lies left-aligned.
			// Resize the free space to begin where our block ends.
			node.borrow_mut().value.start = end;
			return true;
		} else if region_start < address && region_end > end {
			// We found free space that covers the block we want to allocate.
			// Resize the free space to end at our block and add another free space entry that begins where our block ends.
			node.borrow_mut().value.end = address;

			let new_node = unsafe { mm::POOL.list.head().expect("Pool is empty when reserving memory") };
			unsafe { mm::POOL.list.remove(new_node.clone()); }

			{
				let mut new_node_borrowed = new_node.borrow_mut();
				new_node_borrowed.value.start = end;
				new_node_borrowed.value.end = region_end;
			}

			self.list.insert_after(new_node, node);
			return true;
		}

		false
	}

	pub fn allocate_aligned(&mut self, size: usize, alignment: usize) -> Result<usize, ()> {
		debug!("Allocating {} bytes from Free List {:#X} aligned to {} bytes", size, self as *const Self as usize, alignment);

		for node in self.list.iter() {
			// Align up the start address of the current node in the list to the desired alignment.
			// Then let allocate_address_for_node check if this node is suitable and alter it respectively.
			let address = align_up!(node.borrow().value.start, alignment);
			let end = address + size;
			if self.allocate_address_for_node(address, end, node) {
				return Ok(address);
			}
		}

		Err(())
	}

	pub fn reserve(&mut self, address: usize, size: usize) -> Result<(), ()> {
		debug!("Reserving {} bytes at address {:#X} in Free List {:#X}", size, address, self as *const Self as usize);
		let end = address + size;

		for node in self.list.iter() {
			// Let allocate_address_for_node check if this node contains the desired address.
			if self.allocate_address_for_node(address, end, node) {
				return Ok(());
			}
		}

		// Our Free List contains no block covering the given address and size.
		// This is an error, because we have to reserve the address to prevent it from being used differently.
		Err(())
	}

	pub fn deallocate(&mut self, address: usize, size: usize) {
		debug!("Deallocating {} bytes at {:#X} from Free List {:#X}", size, address, self as *const Self as usize);

		let end = address + size;
		let mut iter = self.list.iter();

		while let Some(node) = iter.next() {
			let (region_start, region_end) = {
				let borrowed = node.borrow();
				(borrowed.value.start, borrowed.value.end)
			};

			if region_start == end {
				// The deallocated memory extends this free memory region to the left.
				node.borrow_mut().value.start = address;
				return;
			} else if region_end == address {
				// The deallocated memory extends this free memory region to the right.
				// Check if it can even reunite with the next region.
				if let Some(next_node) = iter.next() {
					let (next_region_start, next_region_end) = {
						let borrowed = node.borrow();
						(borrowed.value.start, borrowed.value.end)
					};

					if next_region_start == end {
						// It can reunite, so let the current region span over the reunited region and move the duplicate node
						// into the pool for deletion or reuse.
						node.borrow_mut().value.end = next_region_end;
						self.list.remove(next_node.clone());
						unsafe { mm::POOL.list.push(next_node); }
						return;
					}
				}

				// It cannot reunite, so just extend this region to the right and we are done.
				node.borrow_mut().value.end = end;
				return;
			} else if end < region_start {
				// The deallocated memory does not extend any memory region and needs an own entry in the Free List.
				// Get that entry from the node pool.
				// We search the list from low to high addresses and insert us before the first entry that has a
				// higher address than us.
				let new_node = unsafe { mm::POOL.list.head().expect("Pool is empty when attempting insert_before") };
				unsafe { mm::POOL.list.remove(new_node.clone()); }

				{
					let mut new_node_borrowed = new_node.borrow_mut();
					new_node_borrowed.value.start = address;
					new_node_borrowed.value.end = end;
				}

				self.list.insert_before(new_node, node);
				return;
			}
		}

		// We could not find an entry with a higher address than us.
		// So we become the new last entry in the list. Get that entry from the node pool.
		let new_node = unsafe { mm::POOL.list.head().expect("Pool is empty when attempting insert_after") };
		unsafe { mm::POOL.list.remove(new_node.clone()); }

		{
			let mut new_node_borrowed = new_node.borrow_mut();
			new_node_borrowed.value.start = address;
			new_node_borrowed.value.end = end;
		}

		if let Some(tail) = self.list.tail() {
			self.list.insert_after(new_node, tail);
		} else {
			self.list.push(new_node);
		}
	}
}
