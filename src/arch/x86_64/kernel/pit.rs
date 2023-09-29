// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::arch::processor::*;
// use crate::consts::*;
use crate::logging::*;
use x86::io::*;
use x86::time::rdtsc;
use core::sync::atomic::{AtomicU64, Ordering};

// const CLOCK_TICK_RATE: u32 = 1193182u32; /* 8254 chip's internal oscillator frequency */

unsafe fn wait_some_time() {
	let start = rdtsc();

	mb();
	while rdtsc() - start < 1000000 {
		mb();
	}
}

static COUNTER: AtomicU64 = AtomicU64::new(0);
/// Ticks are about 1 ms
pub fn get_ticks() -> u64
{
	COUNTER.load(Ordering::Acquire)
}

#[doc(hidden)]
pub fn inc_ticks() -> u64
{
	// Returns the old value
	COUNTER.fetch_add(1, Ordering::AcqRel)
}

// initialize the Programmable Interrupt controller
pub fn init() {
	debug!("initialize timer");

	// 11932
	// let latch = ((CLOCK_TICK_RATE + TIMER_FREQ / 2) / TIMER_FREQ) as u16;
	let latch = 1194u16;

	unsafe {
		/*
		 * Port 0x43 is for initializing the PIT:
		 *
		 * 0x34 means the following:
		 * 0b...     (step-by-step binary representation)
		 * ...  00  - channel 0
		 * ...  11  - write two values to counter register:
		 *            first low-, then high-byte
		 * ... 010  - mode number 2: "rate generator" / frequency divider
		 * ...   0  - binary counter (the alternative is BCD)
		 */
		outb(0x43, 0x34);

		wait_some_time();

		/* Port 0x40 is for the counter register of channel 0 */

		outb(0x40, (latch & 0xFF) as u8); /* low byte  */

		wait_some_time();

		outb(0x40, (latch >> 8) as u8); /* high byte */
	}
}
