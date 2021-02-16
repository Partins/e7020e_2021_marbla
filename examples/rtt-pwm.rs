//! cargo run --examples rtt-pwm

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{gpio::Speed, prelude::*, pwm};

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        rtt_init_print!();
        rprintln!("init");
        let dp = cx.device;

        // Set up the system clock. 16 MHz?
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpioa = dp.GPIOA.split();
        let channels = (
            gpioa.pa8.into_alternate_af1().set_speed(Speed::High),
            gpioa.pa9.into_alternate_af1().set_speed(Speed::High),
        );

        let pwm = pwm::tim1(dp.TIM1, channels, clocks, 1u32.khz());
        let (mut ch1, mut ch2) = pwm;
        let max_duty = ch1.get_max_duty();
        rprintln!("max_duty {}", max_duty);
        ch1.set_duty(max_duty / 2);
        ch1.enable();
        ch2.set_duty((max_duty * 1) / 3);
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
