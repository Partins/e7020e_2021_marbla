//! cargo run --examples rtt-pwm

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use stm32f4xx_hal::{
    gpio::{
        gpiob::{PB8, PB9},
        gpioc::PC13,
        AlternateOD, Edge, ExtiPin, Input, PullUp, Speed, AF4,
    },
    i2c::I2c,
    prelude::*,
    stm32::I2C1,
};

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        rtt_init_print!();
        rprintln!("init");
        let dp = cx.device;

        // Set up the system clock, 48MHz
        let rcc = dp.RCC.constrain();
        // let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
        let clocks = rcc.cfgr.freeze();
        // let clocks = rcc
        //     .cfgr
        //     .hclk(48.mhz())
        //     .sysclk(48.mhz())
        //     .pclk1(24.mhz())
        //     .pclk2(24.mhz())
        //     .freeze();

        // Set up I2C.
        let gpiob = dp.GPIOB.split();
        let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb9.into_alternate_af4().set_open_drain();
        let mut i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks);

        rprintln!("here");

        // configure
        // 7:6 -     reserved
        // 5   ORDER logic 0, the MSB of the data word is transmitted first.
        //           logic 1, the LSB of the data word is transmitted first.
        // 4   -     reserved
        // 3:3 M1:M0 Mode selection
        //           00 - SPICLK LOW when idle; data clocked in on leading edge (CPOL = 0, CPHA = 0)
        //           01 - SPICLK LOW when idle; data clocked in on trailing edge (CPOL = 0, CPHA = 1)
        //           10 - SPICLK HIGH when idle; data clocked in on trailing edge (CPOL = 1, CPHA = 0)
        //           11 - SPICLK HIGH when idle; data clocked in on leading edge (CPOL = 1, CPHA = 1)
        // 1:0 F1:F0 SPI clock rate
        //           00 - 1843 kHz
        //           01 - 461 kHz
        //           10 - 115 kHz
        //           11 - 58 kHz

        let i2c_addr = 0x50 >> 1;
        let i2c_command_conf = 0xF0;

        let i2c_conf_reg = (0b0 << 5) /* MSB First */ |
                           (0b10 << 2) /* Mode 3 */ |
                           (0b00 << 0) /* 1843 kHz */;

        let x = i2c.write(i2c_addr, &[i2c_command_conf, i2c_conf_reg]);
        rprintln!("configure {:?}", x);

        // write to spi with CS0 (command 01..0f)
        let i2c_command_cs0 = 0x01; // bit 0 set
        loop {
            let x = i2c.write(i2c_addr, &[i2c_command_cs0, 0x00, 0xFF, 0xFF, 0xFF]);
            rprintln!("data {:?}", x);
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }
};
