//! rtic_bare6.rs
//!
//! Clocking
//!
//! What it covers:
//! - using svd2rust generated API
//! - using the stm32f4xx-hal to set clocks
//! - routing the clock to a PIN for monitoring by an oscilloscope

#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    prelude::*,
    stm32::{self, GPIOC, RCC},
};

const OFFSET: u32 = 24_000_000;

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

        // setup LED
        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        clock_out(&device.RCC, &device.GPIOC);

        let rcc = device.RCC.constrain();

        //let _clocks = rcc.cfgr.freeze();

        //Set up the system clock. 48 MHz?
        // let _clocks = rcc.cfgr.sysclk(48.mhz()).pclk1(24.mhz()).freeze();

        // 7.1 (panicks at run-time)
        // let _clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze();
        
        // 7.2 
        rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();

        // let _clocks = rcc
        //     .cfgr
        //     .sysclk(64.mhz())
        //     .pclk1(64.mhz())
        //     .pclk2(64.mhz())
        //     .freeze();
        //
        // let _clocks = rcc
        //     .cfgr
        //     .sysclk(84.mhz())
        //     .pclk1(42.mhz())
        //     .pclk2(64.mhz())
        //     .freeze();

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

// see the Reference Manual RM0368 (www.st.com/resource/en/reference_manual/dm00096844.pdf)
// rcc,     chapter 6
// gpio,    chapter 8

fn clock_out(rcc: &RCC, gpioc: &GPIOC) {
    // output MCO2 to pin PC9v

    // mco2 	: SYSCLK = 0b00
    // mcopre 	: divide by 4 = 0b110
    rcc.cfgr
        .modify(|_, w| unsafe { w.mco2().sysclk().mco2pre().div4() });

    // power on GPIOC, RM0368 6.3.11
    rcc.ahb1enr.modify(|_, w| w.gpiocen().enabled());

    // MCO_2 alternate function AF0, STM32F401xD STM32F401xE data sheet
    // table 9
    // AF0, gpioc reset value = AF0

    // configure PC9 as alternate function 0b10, RM0368 6.2.10
    gpioc.moder.modify(|_, w| w.moder9().alternate());

    // otyper reset state push/pull, in reset state (don't need to change)

    // ospeedr 0b11 = very high speed
    gpioc.ospeedr.modify(|_, w| w.ospeedr9().low_speed());
}

// 0. Background reading:
//
//    Clock trees:
//    STM32F401xD STM32F401xE, section 3.11
//    We have two AMBA High-performance Buses (APBs)
//    APB1 low speed bus (max freq 42 MHz)
//    APB2 high speed bus (max freq 84 MHz)
//
//    RM0368 Section 6.2
//    Some important/useful clock acronyms and their use:
//
//    SYSCLK - the clock that drives the `core`
//    HCLK   - the clock that drives the AMBA bus(es), memory, DMA, trace unit, etc.
//
//    Typically we set HCLK = SYSCLK / 1 (no pre-scale) for our applications
//
//    FCLK   - Free running clock running at HCLK
//
//    CST    - CoreSystemTimer drives the SysTick counter, HCLK/(1 or 8)
//    PCLK1  - The clock driving the APB1 (<= 42 MHz)
//             Timers on the APB1 bus will be triggered at PCLK1 * 2
//    PCLK2  - The clock driving the APB2 (<= 84 MHz)
//             Timers on the APB2 bus will be triggered at PCLK2
//
//    Configuration:
//
//    The `Cargo.toml` file defines your dependencies.
//
//    [dependencies.stm32f4]
//    version = "0.13.0"
//    features = ["stm32f411", "rt"]
//
//    [dependencies.stm32f4xx-hal]
//    version = "0.8.3"
//    features = ["rt", "stm32f411", "usb_fs"]
//
//    The stm32f411 and f401 is essentially the same chip, the f411 is guaranteed
//    up to 100MHz, but we can "overclock" the f401 to 100MHz if needed.
//
//    The `features = ["stm32f411", "rt"]` selects the target MCU, and
//    "rt" enables functionality for exception handling etc.
//
//    The HAL provides a generic abstraction over the whole stm32f4 family.
//
//    In our configuration we enable "stm32f411" with the "rt" feature
//    and the "usb_fs" (for USB OnTheGo support).
//
//    The HAL re-exports the selected "stm32f411" under the `stm32` path.
//
//    Initialization:
//
//    In the code, we first setup the DWT/CYCCNT for the Monotonic timer,
//    and schedule a task to be run after `OFFSET` number of clock cycles.
//
//    The `device.RCC.constrain()`, gives a default setting for the MCU RCC
//    (Reset and Clock Control) peripheral.
//    `rcc.cfgr.x.freeze()`, freezes the current (default) config.
//
//    What is wrong with the following configurations?
//
//    `rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze()`;
//
//    PCLK1 is set to 64 MHz but it should not exceed 42 MHz as it's constrained 
//    by the APB1 bus as specified above and in the reference manual page 95. 
//
//    `rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();`
//
//    We can't have a prescaler that's an integer between 84 MHz on the SYSCLK and
//    64 MHz for the APB2 high speed bus. We'd have to set the SYSCLK to 64 MHz
//
//    Start `stm32cubemx` and select or create a project targeting stm32f401.
//    Go to the graphical clock configuration view.
//
//    Try to setup the clock according to:
//
//    `rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze()`;
//
//    What happens?
//
//    It gives an error because PCLK1 freq. cannot exceed 42 MHz
//
//    Try to setup the clock according to:
//
//    What happens?
//
//    `rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();`
//
//    I can't find a prescaler such that I can have a SYSCLK at 84 MHz and PCLK2 at 64 MHz. 
//
//    Commit your answers (bare6_0)
//
// 1. In this example you will use RTT.
//
//    > cargo run --example rtic_bare6
//
//    Confirm that your RTT traces the init, idle and led on/off.
//
//    What is the (default) MCU (SYSCLK) frequency?
//
//    16 MHz
//
//    What is the (default) DWT CYCCNT frequency?
//
//    16 MHz
//
//    What is the frequency of blinking?
//
//    Each toggle is 8_000_000 clock cycles. To blink the LED we need to toggle twice
//    so turning on and off takes 16_000_000 clock cycles. Dividing by the SYSCLK freq.
//    gives us the seconds. Thus the freq. of blinking is 1 Hz.
//
//    Commit your answers (bare6_1)
//
// 2. Now connect an oscilloscope to PC9, which is set to
//    output the MCO2.
//
//    Compute the value of SYSCLK based on the oscilloscope reading
//
//    The oscillicope reading is 4 MHz. Looking at line 122-125 we see that the 
//    MCO2 prescaler is set to 0b110. Looking at p. 107/847 in RM0368 we see 
//    that it corresponds to a division by 4 (also explained by the comment on line 123).
//    So our SYSCLK freq. is 4*4=>16 MHz
//
//    What is the peak to peak (voltage) reading of the signal?
//
//    3.68V
//
//    Make a folder called "pictures" in your git project.
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_16mhz_high_speed".
//
//    Commit your answers (bare6_2)
//
// 3. Now run the example in 48Mz, by commenting out line 56, and un-commenting
//    lines 58-63.
//`
//    What is the frequency of blinking?
//
//    Now it should be 16_000_000 divided by 48_000_000 which is 3 Hz. To get 1 Hz ha have
//    to make the toggle less frequent, specifically 3 times slower -> 8*3 = 24. My new OFFSET
//    is now 24_000_000 cycles
//
//    Now change the constant `OFFSET` so you get the same blinking frequency as in 1.
//    Test and validate that you got the desired behavior.
//
//    Commit your answers (bare6_3)
//
// 4. Repeat experiment 2
//
//    What is the fruency of MCO2 read by the oscilloscope?
//
//    12.14MHz
//
//    Compute the value of SYSCLK based on the oscilloscope reading.
//
//    12.15*4 = 48.6MHz
//
//    What is the peak to peak reading of the signal?
//
//    3.24V
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_48mhz_high_speed".
//
//    Commit your answers (bare6_4)
//
// 5. In the `clock_out` function, the setup of registers is done through
//    setting bit-pattens manually, e.g.
//     rcc.cfgr
//        .modify(|_, w| unsafe { w.mco2().bits(0b00).mco2pre().bits(0b110) });
//
//    However based on the vendor SVD file the svd2rust API provides
//    a better abstraction, based on pattern enums and functions.
//
//    To view the API you can generate documentation for your crate:
//
//    > cargo doc --open
//
//    By searching for `mco2` you find the enumerations and functions.
//    So here
//       `w.mco2().bits{0b00}` is equivalent to
//       `w.mco2().sysclk()` and improves readability.
//
//    Replace all bit-patterns used in `clock_out` by the function name equivalents.
//    (alternatively, use the enum values.)
//
//    Test that the application still runs as before.
//
//    Commit your code (bare6_5)
//
// 6. Now reprogram the PC9 to be "Low Speed", and re-run at 48Mz.
//
//    Did the frequency change in comparison to assignment 5?
//
//    NO. Still at around 12.21 MHz
//
//    What is the peak to peak reading of the signal (and why did it change)?
//
//    3.17V
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_48mhz_low_speed".
//
//    Commit your answers (bare6_6)
//
// 7. Try setting the clocks according to:
//
//    `rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze()`;
//
//    Does the code compile?
//
//    YES.
//
//    What happens at run-time?
//
//    It panicked because we tried to set the pclk1 to 64 MHz but the limit is 48 MHz.
//
//    Try setting the clocks according to:
//
//    `rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();`
//
//    Does the code compile?
//
//    YES
//
//    What happens at run-time?
//
//    It runs but it's not correct as there is no prescaler between SYSCLK 84MHz
//    and PCLK2 64 MHz. If we'd try to set it up with stm32cubemx we would get an error
//    just like in one of the first excercises. 
//
//    Is that a correct?
//
//    Optional: If you find it incorrect, file an issue to `stm32f4xx-hal` describing the problem.
//    (Remember always check already open issues, and add to existing if related.)
//
// 7. Discussion
//
//    In this exercise, you have learned to use the stm32f4xx-hal
//    to set the clock speed of your MCU.
//
//    You have also learned how you can monitor/validate MCU clock(s) on pin(s)
//    connected to an oscilloscope.
//
//    You have also learned how you can improve readability of your code
//    by leveraging the abstractions provided by the PAC.
//
//    As mentioned before the PACs are machine generated by `svd2rust`
//    from vendor provided System View Descriptions (SVDs).
//
//    The PACs provide low level peripheral access abstractions, while
//    the HALs provide higher level abstractions and functionality.
