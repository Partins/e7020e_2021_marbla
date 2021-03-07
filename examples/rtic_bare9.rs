//! bare8.rs
//!
//! Serial
//!
//! What it covers:

#![no_main]
#![no_std]

use panic_rtt_target as _;

use stm32f4xx_hal::{
    prelude::*,
    serial::{config::Config, Event, Rx, Serial, Tx},
    stm32::USART2,
};

use rtic::app;
use rtt_target::{rprintln, rtt_init_print};

#[app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // Late resources
        TX: Tx<USART2>,
        RX: Rx<USART2>,
    }

    // init runs in an interrupt free section
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");

        let device = cx.device;

        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioa = device.GPIOA.split();

        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7();

        let mut serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .unwrap();

        // generate interrupt on Rxne
        serial.listen(Event::Rxne);

        // Separate out the sender and receiver of the serial port
        let (tx, rx) = serial.split();

        // Late resources
        init::LateResources { TX: tx, RX: rx }
    }

    // idle may be interrupted by other interrupts/tasks in the system
    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    // capacity sets the size of the input buffer (# outstanding messages)
    #[task(resources = [TX], priority = 1, capacity = 128)]
    fn rx(cx: rx::Context, data: u8) {
        let tx = cx.resources.TX;
        tx.write(data).unwrap();
        rprintln!("data {}", data);
    }

    // Task bound to the USART2 interrupt.
    #[task(binds = USART2,  priority = 2, resources = [RX], spawn = [rx])]
    fn usart2(cx: usart2::Context) {
        let rx = cx.resources.RX;
        let data = rx.read().unwrap();
        cx.spawn.rx(data).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};

// 0. Background
//
//    As seen in the prior example, you may loose data unless polling frequently enough.
//    Let's try an interrupt driven approach instead.
//
//    In init we just add:
//
//    // generate interrupt on Rxne
//    serial.listen(Event::Rxne);
//
//    This causes the USART hardware to generate an interrupt when data is available.
//
//    // Task bound to the USART2 interrupt.
//    #[task(binds = USART2,  priority = 2, resources = [RX], spawn = [rx])]
//    fn usart2(cx: usart2::Context) {
//      let rx = cx.resources.RX;
//      let data = rx.read().unwrap();
//      cx.spawn.rx(data).unwrap();
//    }
//
//    The `usart2` task will be triggered, and we read one byte from the internal
//    buffer in the USART2 hardware. (panic if something goes bad)
//
//    We send the read byte to the `rx` task (by `cx.spawn.rx(data).unwrap();`)
//    (We panic if the capacity of the message queue is reached)
//
//    // capacity sets the size of the input buffer (# outstanding messages)
//    #[task(resources = [TX], priority = 1, capacity = 128)]
//    fn rx(cx: rx::Context, data: u8) {
//        let tx = cx.resources.TX;
//        tx.write(data).unwrap();
//        rprintln!("data {}", data);
//    }
//
//    Here we echo the data back, `tx.write(data).unwrap();` (panic if usart is busy)
//    We then trace the received data `rprintln!("data {}", data);`
//
//    The `priority = 2` gives the `usart2` task the highest priority
//    (to ensure that we don't miss data).
//
//    The `priority = 1` gives the `rx` task a lower priority.
//    Here we can take our time and process the data.
//
//    `idle` runs at priority 0, lowest priority in the system.
//    Here we can do some background job, when nothing urgent is happening.
//
//    This is an example of a good design!
//
// 1. In this example we use RTT.
//
//    > cargo run --example rtic_bare9
//
//    Try breaking it!!!!
//    Throw any data at it, and see if you could make it panic!
//
//    Were you able to crash it?
//
//    ** your answer here **
//
//    Notice, the input tracing in `moserial` seems broken, and may loose data.
//    So don't be alarmed if data is missing, its a GUI tool after all.
//
//    If you want to sniff the `ttyACM0`, install e.g., `interceptty` and run
//    > interceptty /dev/ttyACM0
//
//    In another terminal, you can do:
//    > cat examples/rtic_bare9.rs > /dev/ttyACM0
//
//    Incoming data will be intercepted/displayed by `interceptty`.
//    (In the RTT trace you will see that data is indeed arriving to the target.)
//
//    Commit your answer (bare9_1)
//
// 2. Now, re-implement the received and error counters from previous exercise.
//
//    Good design:
//    - Defer any tracing to lower priority task/tasks
//      (you may introduce an error task at low priority).
//
//    - State variables can be introduced either locally (static mut), or
//      by using a resource.
//
//      If a resource is shared among tasks of different priorities:
//      The highest priority task will have direct access to the data,
//      the lower priority task(s) will need to lock the resource first.
//
//    Check the RTIC book, https://rtic.rs/0.5/book/en/by-example
//    regarding resources, software tasks, error handling etc.
//
//    Test that your implementation works and traces number of
//    bytes received and errors encountered.
//
//    If implemented correctly, it should be very hard (or impossible)
//    to get an error.
//
//    You can force an error by doing some "stupid delay" (faking workload),
//    e.g., burning clock cycles using `cortex_m::asm::delay` in the
//    `rx` task. Still you need to saturate the capacity (128 bytes).
//
//    To make errors easier to produce, reduce the capacity.
//
//    Once finished, comment your code.
//
//    Commit your code (bare9_2)
//
// 3. Discussion
//
//    Here you have used RTIC to implement a highly efficient and good design.
//
//    Tasks in RTIC are run-to-end, with non-blocking access to resources.
//    (Even `lock` is non-blocking, isn't that sweet?)
//
//    Tasks in RTIC are scheduled according to priorities.
//    (A higher priority task `H` always preempts lower priority task `L` running,
//    unless `L` holds a resource with higher or equal ceiling as `H`.)
//
//    Tasks in RTIC can spawn other tasks.
//    (`capacity` sets the message queue size.)
//
//    By design RTIC guarantees race- and deadlock-free execution.
//
//    It also comes with theoretical underpinning for static analysis.
//    - task response time
//    - overall schedulability
//    - stack memory analysis
//    - etc.
//
//    RTIC leverages on the zero-cost abstractions in Rust,
//    and the implementation offers best in class performance.
