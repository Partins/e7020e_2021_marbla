//! rtic_bare3.rs
//!
//! Measuring execution time
//!
//! What it covers
//! - Reading Rust documentation
//! - Timing abstractions and semantics
//! - Understanding Rust abstractions

#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rtic::cyccnt::Instant;
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(mut cx: init::Context) {
        cx.core.DWT.enable_cycle_counter();

        let start = Instant::now();
        wait(1_000_000);
        let end = Instant::now();

        // notice all printing outside of the section to measure!
        hprintln!("Start {:?}", start).ok();
        hprintln!("End {:?}", end).ok();
        // hprintln!("Diff {:?}", (end - start) ).ok();
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
//    In the docs, search (`S`) for `Monotonic` and read the API docs.
//    Also search for `Instant`, and `Duration`.
//
//    Together these provide timing semantics.
//
//    - `Monotonic` is a "trait" for a timer implementation.
//    - `Instant` is a point in time.
//    - `Duration` is a range in time.
//
//    By default RTIC uses the `Systic` and the `DWT` cycle counter
//    to provide a `Monotonic` timer.
//
// 1. Build and run the application in vscode using (Cortex Release).
//
//    What is the output in the Adapter Output console?
//
//    ** your answer here **
//
//    As you see line 31 is commented out (we never print the difference).
//
//    Now uncomment line 31, and try to run the program. You will see
//    that it fails to compile right as `Duration` does not implement `Debug`
//    (needed for formatting the printout.)
//
//    This is on purpose as `Duration` is abstract (opaque). You need to
//    turn it into a concrete value. Look at the documentation, to find out
//    a way to turn it into clock cycles (which are printable).
//
//    What is now the output in the Adapter Output console?
//
//    ** your answer here **
//
//    Commit your answers (bare3_1)
//
// 2. Look at the `Instant` documentation.
//
//    Alter the code so that you use `duration_since`, instead of manual subtraction.
//
//    What is now the output in the Adapter Output console?
//
//    ** your answer here **
//
//    Commit your answers (bare3_2)
//
// 3. Look at the `Instant` documentation.
//    Now alter the code so that it uses `elapsed` instead.
//
//    What is now the output in the Adapter Output console?
//
//    ** your answer here **
//
//    Commit your answers (bare3_3)
//
// 4. Discussion.
//
//    If you did implement the above exercises correctly you should get exactly the same
//    result (in clock cycles) for all cases as you got in the bare2 exercise.
//    (If not, go back and revise your code.)
//
//    What this shows, is that we can step away from pure hardware accesses
//    and deal with time in a more convenient and "abstract" fashion.
//
//    `Instant` and `Duration` are associated with semantics (meaning).
//    `Monotonic` is associated the implementation.
//
//    This is an example of separation of concerns!
//
//    If you implement your application based on Instant and Duration, your code
//    will be "portable" across all platforms (that implement Monotonic).
//
//    The implementation of Monotonic is done only once for each platform, thus
//    bugs related to low level timer access will occur only at one place,
//    not scattered across thousands of manually written applications.
//
//    However, as you have already seen, the current time abstraction (API) is
//    is rather "thin" (provided just a bare minimum functionality).
//
//    We are working to further generalize timing semantics, by building
//    on a richer abstraction `https://docs.rs/embedded-time/0.10.1/embedded_time/`.
//
//    Support for embedded time is projected for next RTIC release.
