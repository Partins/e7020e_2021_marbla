#![no_main]
#![no_std]

use cortex_m::iprintln;
use panic_halt as _;
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        let mut p = cx.core;
        let stim = &mut p.ITM.stim[0];
        for a in 0..=10 {
            iprintln!(stim, "RTIC Hello, world!! {}", a);
        }
    }
};
