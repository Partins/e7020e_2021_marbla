// cargo run --example rtt_rtic_hello

#![no_main]
#![no_std]

use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(_cx: init::Context) {
        rtt_init_print!();
        for i in 0..11 {
            rprintln!("RTIC Says Hello, world {}!!", i);
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("lets get lazy");
        loop {
            continue;
        }
    }
};
