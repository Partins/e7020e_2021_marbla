#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use embedded_hal::spi::MODE_3;
use panic_rtt_target as _;

use rtic::cyccnt::{Instant, U32Ext as _};
use stm32f4xx_hal::{
    dwt::Dwt,
    gpio::Speed,
    gpio::{
        gpiob::{PB10, PB4},
        gpioc::{PC2, PC3},
        Alternate, Output, PushPull,
    },
    prelude::*,
    rcc::Clocks,
    spi::Spi,
    stm32,
};

use app::{
    pmw3389::{self, Register},
    DwtDelay,
};
use rtt_target::{rprintln, rtt_init_print};

type PMW3389T = pmw3389::Pmw3389<
    Spi<
        stm32f4xx_hal::stm32::SPI2,
        (
            PB10<Alternate<stm32f4xx_hal::gpio::AF5>>,
            PC2<Alternate<stm32f4xx_hal::gpio::AF5>>,
            PC3<Alternate<stm32f4xx_hal::gpio::AF5>>,
        ),
    >,
    PB4<Output<PushPull>>,
>;

#[rtic::app(device = stm32f4xx_hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        pmw3389: PMW3389T,
    }
    #[init(schedule = [poll])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");

        let mut core = cx.core;
        let device = cx.device;

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();

        // setup clocks
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.freeze();
        rprintln!("clocks:");
        rprintln!("hclk {}", clocks.hclk().0);

        // Configure SPI
        // spi2
        // sck    - pb10, (yellow)
        // miso   - pc2, (red)
        // mosi   - pc3, (orange)
        // ncs    - pb4, (long yellow)
        // motion - (brown)
        //
        // +5, (white)
        // gnd, (black)

        let gpiob = device.GPIOB.split();
        let gpioc = device.GPIOC.split();

        let sck = gpiob.pb10.into_alternate_af5();
        let miso = gpioc.pc2.into_alternate_af5();
        let mosi = gpioc.pc3.into_alternate_af5();
        let cs = gpiob.pb4.into_push_pull_output().set_speed(Speed::High);

        let spi = Spi::spi2(
            device.SPI2,
            (sck, miso, mosi),
            MODE_3,
            stm32f4xx_hal::time::KiloHertz(2000).into(),
            clocks,
        );

        let mut delay = DwtDelay::new(&mut core.DWT, clocks);
        let mut pmw3389 = pmw3389::Pmw3389::new(spi, cs, delay).unwrap();

        // set in burst mode
        pmw3389.write_register(Register::MotionBurst, 0x00);

        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = cx.start; // the start time of the system

        cx.schedule.poll(now + 16_000.cycles()).unwrap();

        // pass on late resources
        init::LateResources { pmw3389 }
    }

    #[task(priority = 2, resources = [pmw3389], schedule = [poll], spawn = [trace])]
    fn poll(cx: poll::Context) {
        static mut COUNTER: u32 = 0;
        static mut POS_X: i64 = 0;

        *COUNTER += 1;
        if *COUNTER == 1000 / RATIO {
            cx.spawn.trace(*POS_X).unwrap();
            *COUNTER = 0;
        }

        let (x, _y) = cx.resources.pmw3389.read_status().unwrap();
        *POS_X += x as i64;

        // task should run each second N ms (16_000 cycles at 16MHz)
        cx.schedule
            .poll(cx.scheduled + (RATIO * 16_000).cycles())
            .unwrap();
    }

    #[task(priority = 1)]
    fn trace(_cx: trace::Context, pos: i64) {
        static mut OLD_POS: i64 = 0;
        rprintln!(
            "pos_x {:010}, diff {:010} @{:?}",
            pos,
            pos - *OLD_POS,
            Instant::now()
        );
        *OLD_POS = pos;
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    extern "C" {
        fn EXTI0();
        fn EXTI1();
    }
};

const RATIO: u32 = 5;
