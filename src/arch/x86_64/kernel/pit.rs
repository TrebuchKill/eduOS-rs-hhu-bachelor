// EDIT

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

pub fn busy_sleep(ms: u64)
{
	// While looking at redoxOS-rs read write code, I noticed a busy loop which called a pause function, which on x86(_64) uses _mm_pause
	// May be worth trying out
	let wait_until = get_ticks().checked_add(ms).expect("The OS does not currently support running (and by extension waiting) for this long.");
	loop
	{
		if get_ticks() >= wait_until
		{
			return;
		}
	}
}

// initialize the Programmable Interrupt controller
pub fn init() {
	debug!("initialize timer");

	// 11932
	// let latch = ((CLOCK_TICK_RATE + TIMER_FREQ / 2) / TIMER_FREQ) as u16;
	// The above latch evaluates to 11932. This value gets decremented by one each pit tick and causes an interrupt at value 0 (and a reset of the value).
	// This value with the frequency of CLOCK_TICK_RATE equates to about 10 ms.
	// Dividing by 10 (1193) cause it to be consistently below 1 ms. With 1194 (+1), it is about 1 ms.
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
