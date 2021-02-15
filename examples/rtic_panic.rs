#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;

// Pick one of these panic handlers:

// `panic!` halts execution; the panic message is ignored
// use panic_halt as _;

// Reports panic messages to the host stderr using semihosting
use panic_semihosting as _;

// Logs panic messages using the ITM (Instrumentation Trace Macrocell)
// use panic_itm as _;

use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(_cx: init::Context) {
        for i in 0..10 {
            hprintln!("RTIC Says Hello, world {}!!", 100 / (5 - i)).ok();
        }
    }
};
