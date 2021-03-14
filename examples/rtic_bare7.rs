//! rtic_bare7.rs
//!
//! HAL OutputPin abstractions
//!
//! What it covers:
//! - using embedded hal, and the OutputPin abstraction

#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::stm32;

use stm32f4xx_hal::{
    gpio::{gpioa::PA5, Output, PushPull},
    prelude::*,
};
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};

const OFFSET: u32 = 8_000_000;

#[rtic::app(device = stm32f4xx_hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        // GPIOA: stm32::GPIOA,
        
        led: PA5<Output<PushPull>>,
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
            // Set LED pin PA5 as output
            led: device.GPIOA.split().pa5.into_push_pull_output(),
            //GPIOA: device.GPIOA,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }

    #[task(resources = [led], schedule = [toggle])]
    fn toggle(cx: toggle::Context) {
        static mut TOGGLE: bool = false;
        rprintln!("toggle  @ {:?}", Instant::now());

        // Toggle the led by using the output led resource. 
        if *TOGGLE {
            cx.resources.led.set_high();
            //cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            cx.resources.led.set_low();
            //cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        *TOGGLE = !*TOGGLE;
        cx.schedule.toggle(cx.scheduled + OFFSET.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};

fn _toggle_generic<E>(led: &mut dyn OutputPin<Error = E>, toggle: &mut bool) {
    if *toggle {
        led.set_high().ok();
    } else {
        led.set_low().ok();
    }

    *toggle = !*toggle;
}

fn _toggleable_generic<E>(led: &mut dyn ToggleableOutputPin<Error = E>) {
    led.toggle().ok();
}

// 1. In this example you will use RTT.
//
//    > cargo run --example rtic_bare7
//
//    Look in the generated documentation for `set_high`/`set_low`.
//    (You created documentation for your dependencies in previous exercise
//    so you can just search (press `S`) for `OutputPin`).
//    You will find that these methods are implemented for `Output` pins.
//
//    Now change your code to use these functions instead of the low-level GPIO API.
//
//    HINTS:
//    - A GPIOx peripheral can be `split` into individual PINs Px0..Px15).
//    - A Pxy, can be turned into an `Output` by `into_push_pull_output`.
//    - You may optionally set other pin properties as well (such as `speed`).
//    - An `Output` pin provides `set_low`/`set_high`
//    - Instead of passing `GPIO` resource to the `toggle` task pass the
//      `led: PA5<Output<PushPull>>` resource instead.
//
//    Comment your code to explain the steps taken.
//
//    Confirm that your implementation correctly toggles the LED as in
//    previous exercise.
//
//    Commit your code (bare7_1)
//
// 2. Further generalizations:
//
//    Now look at the documentation for `embedded_hal::digital::v2::OutputPin`.
//
//    You see that the OutputPin trait defines `set_low`/`set_high` functions.
//    Your task is to alter the code to use the `set_low`/`set_high` API.
//
//    The function `_toggle_generic` is generic to any object that
//    implements the `OutputPin<Error = E>` trait.
//
//    Digging deeper we find the type parameter `E`, which in this case
//    is left generic (unbound).
//
//    It will be instantiated with a concrete type argument when called.
//
//    Our `PA5<Output<PushPull>>` implements `OutputPin` trait, thus
//    we can pass the `led` resource to `_toggle_generic`.
//    
//    The error type is given by the stm32f4xx-hal implementation:
//    where `core::convert::Infallible` is used to indicate
//    there are no errors to be expected (hence infallible).
//
//    Additionally, `_toggle_generic` takes a mutable reference
//    `toggle: &mut bool`, so you need to pass your `TOGGLE` variable.
//
//    As you see, `TOGGLE` holds the "state", switching between
//    `true` and `false` (to make your led blink).
//
//    Change your code into using the `_toggle_generic` function.
//    (You may rename it to `toggle_generic` if wished.)
//
//    Confirm that your implementation correctly toggles the LED as in
//    previous exercise.
//
//    Commit your code (bare7_2)
//
// 3. What about the state?
//
//    In your code `TOGGLE` holds the "state". However, the underlying
//    hardware ALSO holds the state (if the corresponding bit is set/cleared).
//
//    What if we can leverage that, and guess what we can!!!!
//
//    Look at the documentation for `embedded_hal::digital::v2::ToggleableOutputPin`,
//    and the implementation of:
//
//    fn _toggleable_generic(led: &mut dyn ToggleableOutputPin<Error = Infallible>) {
//      led.toggle().ok();
//    }
//
//    The latter does not take any state variable, instead it directly `toggle()`
//    the `ToggleableOutputPin`.
//
//    Now alter your code to leverage on the `_toggleable_generic` function.
//    (You should be able to remove the `TOGGLE` state variable altogether.)
//
//    Confirm that your implementation correctly toggles the LED as in
//    previous exercise.
//
//    Commit your code (bare7_3)
//
// 4. Discussion:
//
//    In this exercise you have gone from a very hardware specific implementation,
//    to leveraging abstractions (batteries included).
//
//    Your final code amounts to "configuration" rather than "coding".
//
//    This reduces the risk of errors (as you let the libraries do the heavy lifting).
//
//    This also improves code-re use. E.g., if you were to do something less
//    trivial then merely toggling you can do that in a generic manner,
//    breaking out functionality into "components" re-usable in other applications.
//
//    Of course the example is trivial, you don't gain much here, but the principle
//    is the same behind drivers for USART communication, USB, PMW3389 etc.
//
// 5. More details:
//    
//    Looking closer at the implementation:
//    `led: &mut dyn OutputPin<Error = E>`
//
//    You may ask what kind of mumbo jumbo is at play here.
//
//    This is the way to express that we expect a mutable reference to a trait object 
//    that implements the `OutputPin`. Since we will change the underlying object
//    (in this case an GPIOA pin 5) the reference needs to be mutable.
// 
//    Trait objects are further explained in the Rust book.
//    The `dyn` keyword indicates dynamic dispatch (through a VTABLE).
//    https://doc.rust-lang.org/std/keyword.dyn.html
//
//    Notice: the Rust compiler (rustc + LLVM) is really smart. In many cases
//    it can analyse the call chain, and conclude the exact trait object type at hand.
//    In such cases the dynamic dispatch is turned into a static dispatch
//    and the VTABLE is gone, and we have a zero-cost abstraction.
//
//    If the trait object is stored for e.g., in an array along with other
//    trait objects (of different concrete type), there is usually no telling
//    the concrete type of each element, and we will have dynamic dispatch.
//    Arguably, this is also a zero-cost abstraction, as there is no (obvious)
//    way to implement it more efficiently. Remember, zero-cost is not without cost
//    just that it is as good as it possibly gets (you can't make it better by hand).
//
//    You can also force the compiler to deduce the type at compile time, by using
//    `impl` instead of `dyn`, if you are sure you don't want the compiler to
//    "fallback" to dynamic dispatch.
//
//    You might find Rust to have long compile times. Yes you are right,
//    and this type of deep analysis done in release mode is part of the story.
//    On the other hand, the aggressive optimization allows us to code 
//    in a generic high level fashion and still have excellent performing binaries.