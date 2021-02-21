//! cargo run --examples rtt-pwm

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::asm::delay;
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
                           (0b11 << 2) /* Mode 3 */ |
                           (0b00 << 0) /* 1843 kHz */;
        // (0b01 << 0) /* 461 kHz */;

        let x = i2c
            .write(i2c_addr, &[i2c_command_conf, i2c_conf_reg])
            .unwrap();

        rprintln!("configure {:?}", x);
        // cortex_m::asm::delay(10_000_000);

        // write to spi with CS0 (command 01..0f)
        let i2c_command_cs0 = 0x01; // bit 0 set
        let pmw_command_product_id = 0x00;
        let pmw_command_product_version = 0x01;

        let x = i2c.write(i2c_addr, &[i2c_command_cs0, pmw_command_product_id, 0]);
        rprintln!("request product_id {:?}", x);
        //  cortex_m::asm::delay(10_000_000);

        // read the result
        let mut buff = [0, 0, 0, 0];
        rprintln!("buff {:?}", buff);

        let x = i2c.read(i2c_addr, &mut buff);
        // read the buffer
        cortex_m::asm::delay(100_000);
        rprintln!("data received {:?}", x);
        rprintln!("data received {:?}", buff);

        let x = i2c.write(i2c_addr, &[i2c_command_cs0, pmw_command_product_version, 0]);
        rprintln!("request product_version {:?}", x);
        //  cortex_m::asm::delay(10_000_000);

        // read the result
        let mut buff = [0, 0, 0, 0];
        rprintln!("buff {:?}", buff);

        let x = i2c.read(i2c_addr, &mut buff);
        // read the buffer
        cortex_m::asm::delay(100_000);
        rprintln!("data received {:?}", x);
        rprintln!("data received {:?}", buff);

        // // test of the abstractions

        // use embedded_hal::spi::MODE_3;
        // use SC18IS602::{Order, Speed, SH18IS602};
        // let mut spi_emu =
        //     SH18IS602::new(i2c, 0, Order::MsbFirst, MODE_3, Speed::Speed1843kHz, false);

        // rprintln!("spi_emu initialized");

        // let mut id_request = [0x00];
        // spi_emu.transfer(&mut id_request).unwrap();
        // rprintln!("id_request {:?}", id_request);

        // let mut id_request = [0x00];
        // spi_emu.transfer(&mut id_request).unwrap();
        // rprintln!("response {:?}", id_request);
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {
            continue;
        }
    }
};

// SC18IS602
mod SC18IS602 {
    enum Function {
        SpiReadWrite = 0x00, // 0F..01, where lowest 4 bits are the CSs
        SpiConfigure = 0xF0,
        ClearInterrupt = 0xF1,
        IdleMode = 0xF2,
        GpioWrite = 0xF4,
        GpioRead = 0xF5,
        GpioEnable = 0xF6,
        GpioConfigure = 0xF7,
    }

    impl Function {
        fn id(self) -> u8 {
            self as u8
        }
    }

    pub enum Speed {
        Speed1843kHz = 0b00,
        Speed461kHz = 0b01,
        Speed115kHz = 0b10,
        Speed58kHz = 0b11,
    }

    pub enum Order {
        MsbFirst = 0b0,
        MsbLast = 0b1,
    }

    enum GpioMode {
        QuasiBiDirectional = 0b00,
        PushPull = 0b01,
        InputOnly = 0b10,
        OpenDrain = 0b11,
    }

    impl GpioMode {
        fn val(self) -> u8 {
            self as u8
        }
    }

    use embedded_hal::{
        blocking::{i2c, spi::Transfer},
        digital::v2::OutputPin,
        spi::Mode,
    };

    #[derive(Copy, Clone, Debug)]
    pub enum Error {
        NotConfigured,
    }

    pub struct SH18IS602<I2C>
    where
        I2C: i2c::Write + i2c::Read,
    {
        addr: u8,
        cs: bool,
        i2c: I2C,
        // a backing buffer for shadowing SPI transfers
        buff: [u8; 200],
    }

    use rtt_target::rprintln;
    use Function::*;

    impl<I2C> SH18IS602<I2C>
    where
        I2C: i2c::Write + i2c::Read,
    {
        pub fn new(
            mut i2c: I2C,
            addr: u8,
            order: Order,
            mode: Mode,
            speed: Speed,
            cs: bool,
        ) -> SH18IS602<I2C> {
            let addr = (0x50 + addr) >> 1;
            // set configuration
            let cfg = (order as u8) << 5
                | (mode.polarity as u8) << 3
                | (mode.phase as u8) << 2
                | speed as u8;

            i2c.write(addr, &[SpiConfigure.id(), cfg])
                .map_err(|_| panic!())
                .ok();

            cortex_m::asm::delay(100_000);

            if cs {
                rprintln!("GPIO SS0");
                // Configure SS0 as GPIO
                i2c.write(addr, &[GpioEnable.id(), 0x1])
                    .map_err(|_| panic!())
                    .ok();

                // Configure GPIO SS0 as a PushPull Output
                i2c.write(addr, &[GpioConfigure.id(), GpioMode::PushPull.val()])
                    .map_err(|_| panic!())
                    .ok();
            }
            SH18IS602 {
                addr,
                cs,
                i2c,
                buff: [0; 200],
            }
        }
    }

    impl<I2C> Transfer<u8> for SH18IS602<I2C>
    where
        I2C: i2c::Write + i2c::Read,
    {
        type Error = Error;
        // transfer limited to 200 bytes maximum
        // will panic! if presented larger buffer
        //
        fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
            // initiate a transfer on SS0
            self.buff[0] = 0x01; // SSO write
            self.buff[1..words.len() + 1].clone_from_slice(words);
            // perform the transaction on words.len() + 1 bytes
            // the actual SPI transfer should be words.len()

            rprintln!("write {:?}", &self.buff[0..words.len() + 1]);

            self.i2c
                .write(self.addr, &self.buff[0..words.len() + 1])
                .map_err(|_| panic!())
                .ok();
            cortex_m::asm::delay(100_000);

            self.i2c.read(self.addr, words).map_err(|_| panic!()).ok();
            cortex_m::asm::delay(100_000);

            rprintln!("read {:?}", words);

            Ok(words)
        }
    }

    impl<I2C> OutputPin for SH18IS602<I2C>
    where
        I2C: i2c::Write + i2c::Read,
    {
        type Error = Error;

        fn set_low(&mut self) -> Result<(), Self::Error> {
            if !self.cs {
                Err(Error::NotConfigured)
            } else {
                self.i2c
                    .write(self.addr, &[Function::GpioWrite.id(), 0x0])
                    .map_err(|_| panic!())
                    .ok();
                Ok(())
            }
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            if !self.cs {
                Err(Error::NotConfigured)
            } else {
                self.i2c
                    .write(self.addr, &[Function::GpioWrite.id(), 0x1])
                    .map_err(|_| panic!())
                    .ok();
                Ok(())
            }
        }
    }
}
