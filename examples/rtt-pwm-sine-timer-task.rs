//! cargo run --examples rtt-pwm-sine-timer-task --release

// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::{asm, peripheral::DWT};
use panic_rtt_target as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use rtt_target::{rprint, rprintln, rtt_init_print};

use stm32f4xx_hal::{
    gpio::Speed,
    prelude::*,
    pwm, stm32,
    timer::{Event, Timer},
};

include!(concat!(env!("OUT_DIR"), "/sin_abs_const.rs"));

type Timer2 = Timer<stm32::TIM2>;

#[rtic::app(device = stm32f4xx_hal::stm32,  monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        TIM1: stm32::TIM1,
        timer2: Timer2,
    }
    #[init(schedule = [])]
    fn init(mut cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");
        let dp = cx.device;

        // Initialize (enable) the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();

        let rcc = dp.RCC.constrain();
        // Set up the system clock. 48 MHz?
        let clocks = rcc
            .cfgr
            // .use_hse(8.mhz())
            // .sysclk(48.mhz())
            .sysclk(96.mhz())
            .pclk1(24.mhz())
            .freeze();

        let gpioa = dp.GPIOA.split();
        // we set the pins to VeryHigh to get the sharpest waveform possible
        // (rise and fall times should have similar characteristics)
        let channels = (
            gpioa.pa8.into_alternate_af1().set_speed(Speed::VeryHigh),
            gpioa.pa9.into_alternate_af1().set_speed(Speed::VeryHigh),
        );

        // Setup PWM RAW
        let tim1 = dp.TIM1;
        // Here we need unsafe as we are "stealing" the RCC peripheral
        // At this point it has been contrained into SysConf and used to set clocks
        let rcc = unsafe { &(*stm32::RCC::ptr()) };

        rcc.apb2enr.modify(|_, w| w.tim1en().set_bit());
        rcc.apb2rstr.modify(|_, w| w.tim1rst().set_bit());
        rcc.apb2rstr.modify(|_, w| w.tim1rst().clear_bit());

        // Setup chanel 1 and 2 as pwm_mode1
        tim1.ccmr1_output()
            .modify(|_, w| w.oc1pe().set_bit().oc1m().pwm_mode1());

        tim1.ccmr1_output()
            .modify(|_, w| w.oc2pe().set_bit().oc2m().pwm_mode1());

        // The reference manual is a bit ambiguous about when enabling this bit is really
        // necessary, but since we MUST enable the preload for the output channels then we
        // might as well enable for the auto-reload too
        tim1.cr1.modify(|_, w| w.arpe().set_bit());

        let clk = clocks.pclk2().0 * if clocks.ppre2() == 1 { 1 } else { 2 };
        // check that its actually 48_000_000
        rprintln!("clk {}", clk);

        // we want maximum performance, thus we set the prescaler to 0
        let pre = 0;
        rprintln!("pre {}", pre);
        tim1.psc.write(|w| w.psc().bits(pre));

        // we want 8 bits of resolution
        // so our ARR = 2^8 - 1 = 256 - 1 = 255
        let arr = 255;
        rprintln!("arr {}", arr);
        tim1.arr.write(|w| unsafe { w.bits(arr) });

        //  Trigger update event to load the registers
        tim1.cr1.modify(|_, w| w.urs().set_bit());
        tim1.egr.write(|w| w.ug().set_bit());
        tim1.cr1.modify(|_, w| w.urs().clear_bit());

        // Set main output enable of all Output Compare (OC) registers
        tim1.bdtr.modify(|_, w| w.moe().set_bit());

        // Set output enable for channels 1 and 2
        tim1.ccer.write(|w| w.cc1e().set_bit().cc2e().set_bit());

        // Setup the timer
        tim1.cr1.write(|w| {
            w.cms()
                .bits(0b00) // edge aligned mode
                .dir() // counter used as upcounter
                .clear_bit()
                .opm() // one pulse mode
                .clear_bit()
                .cen() // enable counter
                .set_bit()
        });

        // Set main output enable of all Output Compare (OC) registers
        tim1.bdtr.modify(|_, w| w.moe().set_bit());

        // Set duty cycle of Channels
        tim1.ccr1.write(|w| unsafe { w.ccr().bits(128) });
        tim1.ccr2.write(|w| unsafe { w.ccr().bits(128) });

        // Set preload for the CCx
        tim1.cr2.write(|w| w.ccpc().set_bit());

        // Enable update events
        tim1.dier.write(|w| w.uie().enabled());
        tim1.sr.modify(|_, w| w.uif().clear());

        // Set divider to 4, (48_000_000/256)/4
        // tim1.rcr.modify(|_, w| unsafe { w.rep().bits(4) });

        while tim1.sr.read().uif().is_clear() {
            rprint!("-");
        }
        rprintln!("here");
        tim1.sr.modify(|_, w| w.uif().clear());

        let mut tim2: Timer<stm32::TIM2> = Timer::tim2(dp.TIM2, 48000, clocks);
        tim2.listen(Event::TimeOut);
        init::LateResources {
            TIM1: tim1,
            timer2: tim2,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        // panic!("panic");
        loop {
            rprintln!("-");
            continue;
        }
    }

    #[task(binds = TIM2, resources = [TIM1, timer2])]
    fn tim2(cx: tim2::Context) {
        static mut INDEX: u16 = 0;
        static mut LEFT: u16 = 0;
        static mut RIGHT: u16 = 0;
        cx.resources.timer2.clear_interrupt(Event::TimeOut);

        let tim1 = cx.resources.TIM1;

        tim1.ccr1.write(|w| unsafe { w.ccr().bits(*LEFT) });
        tim1.ccr2.write(|w| unsafe { w.ccr().bits(*RIGHT) });

        *INDEX = (*INDEX).wrapping_add(10_000);

        *LEFT = SINE_BUF[*INDEX as usize] as u16;
        *RIGHT = SINE_BUF[*INDEX as usize] as u16;
    }

    extern "C" {
        fn EXTI0();
    }
};

// We aim for a sampling rate of 48kHz, assuming that the input filter of the
// sound card used to sample the generated signal has an appropriate input filter
const PERIOD: u32 = 1000; // 48_000_000 / 48_000;
