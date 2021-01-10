//! examples/rtt_timing.rs
//! cargo run --examples rtt-pwm

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::{asm, peripheral::DWT};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{dma, gpio::Speed, prelude::*, pwm, stm32};

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    #[init]
    fn init(mut cx: init::Context) {
        rtt_init_print!();
        rprintln!("init");
        let dp = cx.device;

        // Initialize (enable) the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();

        // Set up the system clock. 16 MHz?
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpioa = dp.GPIOA.split();
        let channels = (
            gpioa.pa8.into_alternate_af1().set_speed(Speed::Low),
            gpioa.pa9.into_alternate_af1(),
        );

        let pwm = pwm::tim1(dp.TIM1, channels, clocks, 1u32.khz());
        let (mut ch1, mut ch2) = pwm;
        let max_duty = ch1.get_max_duty();
        rprintln!("max_duty {}", max_duty);
        ch1.set_duty(max_duty / 2);
        ch1.enable();
        ch2.set_duty((max_duty * 1) / 2);
        ch2.enable();
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }
};
