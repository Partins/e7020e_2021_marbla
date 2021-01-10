//! examples/rtt_timing.rs
//! cargo run --examples rtt-timing

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::{asm, peripheral::DWT};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(mut cx: init::Context) {
        rtt_init_print!();
        rprintln!("init");

        // Initialize (enable) the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();

        rprintln!("start timed_loop");
        let (start, end) = timed_loop();
        rprintln!(
            "start {}, end {}, diff {}",
            start,
            end,
            end.wrapping_sub(start)
        );
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }
};

// Forbid inlining and keeping name (symbol) readable.
#[inline(never)]
#[no_mangle]
fn timed_loop() -> (u32, u32) {
    let start = DWT::get_cycle_count();
    for _ in 0..10000 {
        asm::nop();
    }
    let end = DWT::get_cycle_count();
    (start, end)
}
