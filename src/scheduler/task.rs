// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
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

#![allow(dead_code)]

use alloc;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt;
use alloc::alloc::{alloc, dealloc, Layout};
use arch;
use arch::processor::msb;
use arch::{PageSize,BasePageSize};
use logging::*;
use consts::*;

extern {
    fn get_bootstack() -> *mut u8;
}

/// The status of the task - used for scheduling
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
	TaskInvalid,
	TaskReady,
	TaskRunning,
	TaskBlocked,
	TaskFinished,
	TaskIdle
}

/// Unique identifier for a task (i.e. `pid`).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskId(u32);

impl TaskId {
	pub const fn into(self) -> u32 {
		self.0
	}

	pub const fn from(x: u32) -> Self {
		TaskId(x)
	}
}

impl alloc::fmt::Display for TaskId {
	fn fmt(&self, f: &mut fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Priority of a task
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskPriority(u8);

impl TaskPriority {
	pub const fn into(self) -> u8 {
		self.0
	}

	pub const fn from(x: u8) -> Self {
		TaskPriority(x)
	}
}

impl alloc::fmt::Display for TaskPriority {
	fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

pub const REALTIME_PRIORITY: TaskPriority = TaskPriority::from(NO_PRIORITIES as u8 - 1);
pub const HIGH_PRIORITY: TaskPriority = TaskPriority::from(24);
pub const NORMAL_PRIORITY: TaskPriority = TaskPriority::from(16);
pub const LOW_PRIORITY: TaskPriority = TaskPriority::from(0);

struct QueueHead {
	head: Option<Rc<RefCell<Task>>>,
	tail: Option<Rc<RefCell<Task>>>,
}

impl QueueHead {
	pub const fn new() -> Self {
		QueueHead {
			head: None,
			tail: None
		}
	}
}

impl Default for QueueHead {
	fn default() -> Self {
		Self { head: None, tail: None }
	}
}

/// Realize a priority queue for tasks
pub struct PriorityTaskQueue {
	queues: [QueueHead; NO_PRIORITIES],
	prio_bitmap: u64
}

impl PriorityTaskQueue {
	/// Creates an empty priority queue for tasks
	pub fn new() -> PriorityTaskQueue {
		PriorityTaskQueue {
			queues: Default::default(),
			prio_bitmap: 0
		}
	}

	/// Add a task by its priority to the queue
	pub fn push(&mut self, task: Rc<RefCell<Task>>) {
		let i = task.borrow().prio.into() as usize;
		//assert!(i < NO_PRIORITIES, "Priority {} is too high", i);

		self.prio_bitmap |= 1 << i;
		match self.queues[i].tail {
			None => {
				// first element in the queue
				self.queues[i].head = Some(task.clone());

				let mut borrow = task.borrow_mut();
					borrow.next = None;
				borrow.prev = None;
			},
			Some(ref mut tail) => {
				// add task at the end of the node
				tail.borrow_mut().next = Some(task.clone());

				let mut borrow = task.borrow_mut();
				borrow.next = None;
				borrow.prev = Some(tail.clone());
			}
		}

		self.queues[i].tail = Some(task.clone());
	}

	fn pop_from_queue(&mut self, queue_index: usize) -> Option<Rc<RefCell<Task>>> {
		let new_head;
		let task;

		match self.queues[queue_index].head {
			None => { return None; },
			Some(ref mut head) => {
				let mut borrow = head.borrow_mut();

				match borrow.next {
					Some(ref mut nhead) => { nhead.borrow_mut().prev = None; },
					None => {}
				}

				new_head = borrow.next.clone();
				borrow.next = None;
				borrow.prev = None;

				task = head.clone();
			}
		}

		self.queues[queue_index].head = new_head;
		if self.queues[queue_index].head.is_none() {
			self.queues[queue_index].tail = None;
			self.prio_bitmap &= !(1 << queue_index as u64);
		}

		Some(task)
	}

	/// Pop the task with the highest priority from the queue
	pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = msb(self.prio_bitmap) {
			return self.pop_from_queue(i as usize);
		}

		None
	}

	/// Pop the next task, which has a higher or the same priority as `prio`
	pub fn pop_with_prio(&mut self, prio: TaskPriority) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = msb(self.prio_bitmap) {
			if i >= prio.into() as u64 {
				return self.pop_from_queue(i as usize);
			}
		}

		None
	}

	/// Remove a specific task from the priority queue.
	pub fn remove(&mut self, task: Rc<RefCell<Task>>) {
		let i = task.borrow().prio.into() as usize;
		//assert!(i < NO_PRIORITIES, "Priority {} is too high", i);

		let mut curr = self.queues[i].head.clone();
		let mut next_curr;

		loop {
			match curr {
				None => { break; },
				Some(ref curr_task) => {
					if Rc::ptr_eq(&curr_task, &task) {
						let (mut prev, mut next) = {
							let mut borrowed = curr_task.borrow_mut();
							(borrowed.prev.clone(), borrowed.next.clone())
						};

						match prev {
							Some(ref mut t) => { t.borrow_mut().next = next.clone(); },
							None => {}
						};

						match next {
							Some(ref mut t) => { t.borrow_mut().prev = prev.clone(); },
							None => {}
						};

						break;
					}

					next_curr = curr_task.borrow().next.clone();
				}
			}

			curr = next_curr.clone();
		}

		let new_head = match self.queues[i].head {
			Some(ref curr_task) => {
				if Rc::ptr_eq(&curr_task, &task) {
					true
				} else {
					false
				}
			},
			None => { false }
		};

		if new_head == true {
				self.queues[i].head = task.borrow().next.clone();

				if self.queues[i].head.is_none() {
					self.prio_bitmap &= !(1 << i as u64);
				}
		}
	}
}

#[derive(Copy, Clone)]
#[repr(align(64))]
#[repr(C)]
pub struct Stack {
	buffer: [u8; STACK_SIZE]
}

impl Stack {
	pub const fn new() -> Stack {
		Stack {
			buffer: [0; STACK_SIZE]
		}
	}

	pub fn top(&self) -> usize {
		(&(self.buffer[STACK_SIZE - 16]) as *const _) as usize
	}

	pub fn bottom(&self) -> usize {
		(&(self.buffer[0]) as *const _) as usize
	}
}

pub static mut BOOT_STACK: Stack = Stack::new();

/// A task control block, which identifies either a process or a thread
#[repr(align(64))]
pub struct Task {
	/// The ID of this context
	pub id: TaskId,
	/// Task Priority
	pub prio: TaskPriority,
	/// Status of a task, e.g. if the task is ready or blocked
	pub status: TaskStatus,
	/// Last stack pointer before a context switch to another task
	pub last_stack_pointer: usize,
	// Stack of the task
	pub stack: *mut Stack,
	// Physical address of the 1st level page table
	pub root_page_table: usize,
	// next task in queue
	pub next: Option<Rc<RefCell<Task>>>,
	// previous task in queue
	pub prev: Option<Rc<RefCell<Task>>>
}

impl Task {
	pub fn new_idle(id: TaskId) -> Task {
		Task {
			id: id,
			prio: LOW_PRIORITY,
			status: TaskStatus::TaskIdle,
			last_stack_pointer: 0,
			stack: unsafe { &mut BOOT_STACK },
			root_page_table: arch::get_kernel_root_page_table(),
			next: None,
			prev: None
		}
	}

	pub fn new(id: TaskId, status: TaskStatus, prio: TaskPriority) -> Task {
		let stack = unsafe { alloc(Layout::new::<Stack>()) as *mut Stack };

		debug!("Allocate stack for task {} at 0x{:x}", id, stack as usize);

		Task {
			id: id,
			prio: prio,
			status: status,
			last_stack_pointer: 0,
			stack: stack,
			root_page_table: arch::get_kernel_root_page_table(),
			next: None,
			prev: None
		}
	}
}

pub trait TaskFrame {
	/// Create the initial stack frame for a new task
	fn create_stack_frame(&mut self, func: extern fn());
}

impl Drop for Task {
	fn drop(&mut self) {
		if unsafe { self.stack != &mut BOOT_STACK } {
			debug!("Deallocate stack of task {} (stack at 0x{:x})", self.id, self.stack as usize);

			// deallocate stack
			unsafe { dealloc(self.stack as *mut u8, Layout::new::<Stack>()); }
		}

		if self.root_page_table != arch::get_kernel_root_page_table() {
			debug!("Deallocate page table 0x{:x} of task {}", self.root_page_table, self.id);
			arch::mm::physicalmem::deallocate(self.root_page_table, BasePageSize::SIZE);
		}
	}
}
