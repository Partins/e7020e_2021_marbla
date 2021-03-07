//! bare8.rs
//!
//! Serial
//!
//! What it covers:
//! - serial communication
//! - bad design

#![no_main]
#![no_std]

use panic_rtt_target as _;

use nb::block;

use stm32f4xx_hal::{
    gpio::{gpioa::PA, Output, PushPull},
    prelude::*,
    serial::{config::Config, Rx, Serial, Tx},
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

        let serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .unwrap();

        // Separate out the sender and receiver of the serial port
        let (tx, rx) = serial.split();

        // Late resources
        init::LateResources { TX: tx, RX: rx }
    }

    // idle may be interrupted by other interrupts/tasks in the system
    #[idle(resources = [RX, TX])]
    fn idle(cx: idle::Context) -> ! {
        let rx = cx.resources.RX;
        let tx = cx.resources.TX;

        loop {
            match block!(rx.read()) {
                Ok(byte) => {
                    rprintln!("Ok {:?}", byte);
                    tx.write(byte).unwrap();
                }
                Err(err) => {
                    rprintln!("Error {:?}", err);
                }
            }
        }
    }
};

// 0. Background
//
//    The Nucleo st-link programmer provides a Virtual Com Port (VCP).
//    It is connected to the PA2(TX)/PA3(RX) pins of the stm32f401/411.
//    On the host, the VCP is presented under `/dev/ttyACMx`, where
//    `x` is an enumerated number (ff 0 is busy it will pick 1, etc.)
//
// 1. In this example we use RTT.
//
//    > cargo run --example rtic_bare8
//
//    Start a terminal program, e.g., `moserial`.
//    Connect to the port
//
//    Device       /dev/ttyACM0
//    Baude Rate   115200
//    Data Bits    8
//    Stop Bits    1
//    Parity       None
//
//    This setting is typically abbreviated as 115200 8N1.
//
//    Send a single character (byte), (set the option `No end` in `moserial`).
//    Verify that sent bytes are echoed back, and that RTT tracing is working.
//
//    Try sending "a", don't send the quotation marks, just a.
//
//    What do you receive in `moserial`?
//
//    ** your answer here **
//
//    What do you receive in the RTT terminal?
//
//    ** your answer here **
//
//    Try sending: "abcd" as a single sequence, don't send the quotation marks, just abcd.
//
//    What did you receive in `moserial`?
//
//    ** your answer here **
//
//    What do you receive in the RTT terminal?
//
//    ** your answer here **
//
//    What do you believe to be the problem?
//
//    Hint: Look at the code in `idle` what does it do?
//
//    ** your answer here **
//
//    Experiment a bit, what is the max length sequence you can receive without errors?
//
//    ** your answer here **
//
//    Commit your answers (bare8_1)
//
// 2. Add a local variable `received` that counts the number of bytes received.
//    Add a local variable `errors` that counts the number of errors.
//
//    Adjust the RTT trace to print the added information inside the loop.
//
//    Compile/run reconnect, and verify that it works as intended.
//
//    Commit your development (bare8_2)
//
// 3. Experiment a bit, what is the max length sequence you can receive without errors?
//
//    ** your answer here **
//
//    How did the added tracing/instrumentation affect the behavior?
//
//    ** your answer here **
//
//    Commit your answer (bare8_3)
//
// 4. Now try compile and run the same experiment 3 but in --release mode.
//
//    > cargo run --example rtic_bare8 --release
//
//    Reconnect your `moserial` terminal.
//
//    Experiment a bit, what is the max length sequence you can receive without errors?
//
//    ** your answer here **
//
//    Commit your answer (bare8_4)
//
// 5. Discussion
//
//    (If you ever used Arduino, you might feel at home with the `loop` and poll design.)
//
//    Typically, this is what you can expect from a polling approach, if you
//    are not very careful what you are doing. This exemplifies a bad design.
//
//    Loss of data might be Ok for some applications but this typically NOT what we want.
//
//    (With that said, Arduino gets away with some simple examples as their drivers do
//    internal magic - buffering data etc.)
