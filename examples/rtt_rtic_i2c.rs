//! cargo run --examples rtt-pwm

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::{asm::delay, delay};
// use panic_halt as _;
use panic_rtt_target as _;
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

use app::{
    pmw3389e::{self, Register},
    DwtDelay,
};

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        rtt_init_print!();
        rprintln!("init");
        let dp = cx.device;
        let mut cp = cx.core;

        // Set up the system clock
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        // Initialize (enable) the monotonic timer (CYCCNT)
        cp.DCB.enable_trace();

        // Set up I2C.
        let gpiob = dp.GPIOB.split();
        let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb9.into_alternate_af4().set_open_drain();
        let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks);

        rprintln!("i2c configured");

        use embedded_hal::spi::MODE_3;
        use SC18IS602::{Order, Speed, SH18IS602};
        let mut spi_emu =
            SH18IS602::new(i2c, 0, Order::MsbFirst, MODE_3, Speed::Speed1843kHz, true);

        rprintln!("spi_emu initialized");

        // reset SPI transfer
        spi_emu.set_low().ok();
        cortex_m::asm::delay(1_000_000);
        spi_emu.set_high().ok();
        cortex_m::asm::delay(1_000_000);

        rprintln!("set to gpio management");

        // try split transaction
        rprintln!("try split transaction");
        // the write part
        spi_emu.set_low().unwrap();
        let mut req = [0x00];
        spi_emu.transfer(&mut req).unwrap();
        rprintln!("id request {:02x?}", req);

        cortex_m::asm::delay(1_000);
        // the read part
        let mut req = [00];
        spi_emu.transfer(&mut req).unwrap();
        rprintln!("id resp {:02x?}", req);

        spi_emu.set_high().unwrap();

        rprintln!("try split transaction");
        // the write part
        spi_emu.set_low().unwrap();
        let mut req = [0x01];
        spi_emu.transfer(&mut req).unwrap();
        rprintln!("version request {:02x?}", req);

        cortex_m::asm::delay(1_000);
        // the read part
        let mut req = [00];
        spi_emu.transfer(&mut req).unwrap();
        rprintln!("version resp {:02x?}", req);

        spi_emu.set_high().unwrap();

        let delay = DwtDelay::new(&mut cp.DWT, clocks);
        let pmw3389 = pmw3389e::Pmw3389e::new(spi_emu, delay).unwrap();

        rprintln!("success");
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
    pub enum Function {
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
        pub fn id(self) -> u8 {
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
        gpio: bool,
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
            i2c: I2C,
            addr: u8,
            order: Order,
            mode: Mode,
            speed: Speed,
            gpio: bool,
        ) -> SH18IS602<I2C> {
            let addr = (0x50 + addr) >> 1;
            // set configuration
            let mut device = SH18IS602 {
                addr,
                gpio,
                i2c,
                buff: [0; 200],
            };

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

            let cfg = (order as u8) << 5
                | (mode.polarity as u8) << 3
                | (mode.phase as u8) << 2
                | speed as u8;

            device.i2c.write(addr, &mut [SpiConfigure.id(), cfg]).ok();

            if gpio {
                device.set_ss0_gpio();
            } else {
                device.set_ss0_hw();
            }

            device
        }

        pub fn set_ss0_gpio(&mut self) {
            rprintln!("SSO as GPIO");
            // Configure SS0 as GPIO
            let buff = [GpioEnable.id(), 0x1];
            rprintln!("GPIO SS0 Enable {:02x?}", buff);
            self.i2c.write(self.addr, &mut [GpioEnable.id(), 0x1]).ok();

            // Configure GPIO SS0 as a PushPull Output
            let buff = [GpioConfigure.id(), GpioMode::PushPull.val()];
            rprintln!("GPIO SS0 PushPull {:02x?}", buff);
            self.i2c.write(self.addr, &buff).ok();

            // Set the SS0 to high out of transaction (idle)
            self.gpio = true;
            self.set_high().unwrap();
        }

        pub fn set_ss0_hw(&mut self) {
            // Configure SS0 as managed by HW
            rprintln!("GPIO SS0 managed by HW");
            self.i2c.write(self.addr, &mut [GpioEnable.id(), 0x0]).ok();
            self.gpio = false;
        }
    }

    // impl<I2C> Default for SH18IS602<I2C> where I2C: i2c::Write + i2c::Read {}

    impl<I2C> Transfer<u8> for SH18IS602<I2C>
    where
        I2C: i2c::Write + i2c::Read,
    {
        type Error = Error;
        // Notice: Transfer limited to 200 bytes maximum
        // panic!  if presented larger buffer
        fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
            // initiate a transfer on SS0
            self.buff[0] = if self.gpio {
                // Due to REASONS UNKNOWN, the SPI is disabled if
                // the corresponding SSx is configured as gpio
                0x02
            } else {
                0x01
            };
            self.buff[0] = 0x02; // SSO write
            self.buff[1..words.len() + 1].clone_from_slice(words);
            // perform the transaction on words.len() + 1 bytes
            // the actual SPI transfer should be words.len()

            // rprintln!("transfer_write {:02x?}", &self.buff[0..words.len() + 1]);

            self.i2c
                .write(self.addr, &self.buff[0..words.len() + 1])
                .map_err(|_| panic!())
                .ok();

            // A short delay is needed
            // For improved performance use write if result is not needed
            cortex_m::asm::delay(1000);

            self.i2c.read(self.addr, words).map_err(|_| panic!()).ok();

            // rprintln!("transfer_read {:02x?}", words);

            Ok(words)
        }
    }

    impl<I2C> OutputPin for SH18IS602<I2C>
    where
        I2C: i2c::Write + i2c::Read,
    {
        type Error = Error;

        fn set_low(&mut self) -> Result<(), Self::Error> {
            if !self.gpio {
                Err(Error::NotConfigured)
            } else {
                rprintln!("set low");
                self.i2c
                    .write(self.addr, &[Function::GpioWrite.id(), 0x0])
                    .map_err(|_| panic!())
                    .ok();
                cortex_m::asm::delay(100_000);
                Ok(())
            }
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            if !self.gpio {
                Err(Error::NotConfigured)
            } else {
                rprintln!("set_high");
                self.i2c
                    .write(self.addr, &[Function::GpioWrite.id(), 0x1])
                    .map_err(|_| panic!())
                    .ok();
                Ok(())
            }
        }
    }
}
