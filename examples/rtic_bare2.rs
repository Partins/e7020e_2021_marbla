//! rtic_bare2.rs
//!
//! Measuring execution time
//!
//! What it covers
//! - Generating documentation
//! - Using core peripherals
//! - Measuring time using the DWT

#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(mut cx: init::Context) {
        cx.core.DWT.enable_cycle_counter();

        // Reading the cycle counter can be done without `owning` access
        // the DWT (since it has no side effect).
        //
        // Look in the docs:
        // pub fn enable_cycle_counter(&mut self)
        // pub fn get_cycle_count() -> u32
        //
        // Notice the difference in the function signature!

        let start = DWT::get_cycle_count();
        wait(1_000_000);
        let end = DWT::get_cycle_count();

        // notice all printing outside of the section to measure!
        hprintln!("Start {:?}", start).ok();
        hprintln!("End {:?}", end).ok();
        hprintln!("Diff {:?}", end.wrapping_sub(start)).ok();

        // wait(100);
    }
};

// burns CPU cycles by just looping `i` times
#[inline(never)]
#[no_mangle]
fn wait(i: u32) {
    for _ in 0..i {
        // no operation (ensured not optimized out)
        cortex_m::asm::nop();
    }
}

// 0. Setup
//
//    > cargo doc --open
//
//    `cargo.doc` will document your crate, and open the docs in your browser.
//    If it does not auto-open, then copy paste the path shown in your browser.
//
//    Notice, it will try to document all dependencies, you may have only one
//    one panic handler, so temporarily comment out all but one in `Cargo.toml`.
//
//    In the docs, search (`S`) for DWT, and click `cortex_m::peripheral::DWT`.
//    Read the API docs.
//
// 1. Build and run the application in vscode using (Cortex Debug).
//
//    What is the output in the Adapter Output console?
//    (Notice, it will take a while we loop one million times at only 16 MHz.)
//
//    ** your answer here **
//
//    Rebuild and run in (Cortex Release).
//
//    ** your answer here **
//
//    Compute the ratio between debug/release optimized code
//    (the speedup).
//
//    ** your answer here **
//
//    commit your answers (bare2_1)
//
// 2. As seen there is a HUGE difference in between Debug and Release builds.
//    In Debug builds, the compiler preserves all abstractions, so there will
//    be a lot of calls and pointer indirections.
//
//    In Release builds, the compiler strives to "smash" all abstractions into straight
//    line code.
//
//    This is what Rust "zero-cost abstractions" means, not zero execution time but rather,
//    "as good as it possibly gets" (you pay no extra cost for using abstractions at run-time).
//
//    In Release builds, the compiler is able to "specialize" the implementation
//    of each function.
//
//    Let us look in detail at the `wait` function:
//    Place a breakpoint at line 54 (wait). Restart the (Cortex Release) session and
//    look at the generated code.
//
//    > disass
//
//    Dump generated assembly for the "wait" function.
//
//    ** your answer here **
//
//    Under the ARM calling convention, r0.. is used as arguments.
//    However in this case, we se that r0 is set by the assembly instructions,
//    before the loop is entered.
//
//    Lookup the two instructions `movw` and `movt` to figure out what happens here.
//
//    Answer in your own words, how they assign r0 to 1000000.
//
//    ** your answer here **
//
//    Commit your answers (bare2_2)
//
// 3. Now add a second call to `wait` (line 42).
//
//    Recompile and run until the breakpoint.
//
//    Dump the generated assembly for the "wait" function.
//
//    ** your answer here **
//
//    Answer in your own words, why you believe the generated code differs?
//
//    ** your answer here **
//
//    Commit your answers (bare2_3)
