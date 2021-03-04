//! rtic_bare7.rs
//!
//! Clocking
//!
//! What it covers:
//! - using embedded hal, and the OutputPin abstraction

use panic_rtt_target as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::stm32;

const OFFSET: u32 = 8_000_000;

#[rtic::app(device = stm32f4xx_hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        GPIOA: stm32::GPIOA,
    }
    #[init(schedule = [toggle])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");

        let mut core = cx.core;
        let device = cx.device;

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();

        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = cx.start; // the start time of the system

        // Schedule `toggle` to run 8e6 cycles (clock cycles) in the future
        cx.schedule.toggle(now + OFFSET.cycles()).unwrap();

        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        // pass on late resources
        init::LateResources {
            GPIOA: device.GPIOA,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }

    #[task(resources = [GPIOA], schedule = [toggle])]
    fn toggle(cx: toggle::Context) {
        static mut TOGGLE: bool = false;
        rprintln!("toggle  @ {:?}", Instant::now());

        if *TOGGLE {
            cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        *TOGGLE = !*TOGGLE;
        cx.schedule.toggle(cx.scheduled + OFFSET.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};

// 1. In this example you will use RTT.
//
//    > cargo run --example rtic_bare7
//
//    Now look at the documentation for `embedded_hal::digital::v2::OutputPin`.
//    (You created documentation for your dependencies in previous exercise
//    so you can just search (press `S`) for `OutputPin`).
//
//    You see that the OutputPin trait defines `set_low`/`set_high` functions.
//    Your task is to alter the code to use the `set_low`/`set_high` API.
//
//    HINTS:
//    - A GPIOx peripheral can be `split` into individual PINs Px0..Px15).
//    - A Pxy, can be turned into an `Output` by `into_push_pull_output`.
//    - You may optionally set other pin properties as well (such as `speed`).
//    - An `Output` pin provides `set_low`/`set_high`
//      (and implements the `OutputPin` trait in embedded-hal).
//
//    Comment your code to explain the steps taken.
//
//    Confirm that your implementation correctly toggles the LED as in
//    previous exercise.
//
//    Commit your code (bare7_1)
//
// 2. Optional
//
//    Use the `toggle` function instead to further simply your code.
//
//    Notice:
//    The `ToggleableOutputPin` abstraction requires `embedded-hal`
//    to compiled with the `unproven` feature.
//
//    The `embedded-hal` traits is mostly used to write drivers
//    that is hardware agnostic (and thus cross platform).
//
//    However:
//    In our case we can use `toggle` directly as implemented by the `stm32f4xx-hal`.
//
//    Confirm that your implementation correctly toggles the LED.
//
//    Which one do you prefer and why (what problem does it solve)?
//
//    ** your answer here **
//
//    Commit your answer (bare7_2)
//
// 3. Discussion
//
//    In this exercise you have learned more on navigating the generated documentation
//    and to use abstractions to simplify and generalize your code.
