//! examples/rtt-pwm-sine.rs
//! cargo run --examples rtt-pwm-sine --release

// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use core::f32::consts::PI;
use cortex_m::{asm, peripheral::DWT};
use panic_halt as _;
use rtt_target::{rprint, rprintln, rtt_init_print};

use stm32f4xx_hal::{bb, dma, gpio::Speed, prelude::*, pwm, stm32};

include!(concat!(env!("OUT_DIR"), "/sin_abs_const.rs"));

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

        let rcc = dp.RCC.constrain();
        // Set up the system clock. 48 MHz?
        let clocks = rcc
            .cfgr
            // .use_hse(8.mhz())
            .sysclk(48.mhz())
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

        tim1.dier.write(|w| w.uie().enabled());
        tim1.sr.modify(|_, w| w.uif().clear());

        // Set divider to 4, (48_000_000/256)/4
        tim1.rcr.modify(|_, w| unsafe { w.rep().bits(4) });

        while tim1.sr.read().uif().is_clear() {
            rprint!("-");
        }
        rprintln!("here");
        tim1.sr.modify(|_, w| w.uif().clear());

        loop {
            for i in 0..SINE_BUF_SIZE {
                // wait until next update event

                while tim1.sr.read().uif().is_clear() {}
                tim1.sr.modify(|_, w| w.uif().clear());

                tim1.ccr1
                    .write(|w| unsafe { w.ccr().bits(SINE_BUF[i] as u16) });
                tim1.ccr2
                    .write(|w| unsafe { w.ccr().bits(SINE_BUF[i] as u16) });
            }
        }
    }

    // [task(resources = [GPIOA], schedule = [toggle])]
    // fn toggle(cx: toggle::Context) {
    //     static mut TOGGLE: bool = false;
    //     hprintln!("foo  @ {:?}", Instant::now()).unwrap();

    //     if *TOGGLE {
    //         cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
    //     } else {
    //         cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
    //     }

    //     *TOGGLE = !*TOGGLE;
    //     cx.schedule
    //         .toggle(cx.scheduled + 8_000_000.cycles())
    //         .unwrap();
    // }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }
};
