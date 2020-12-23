#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::{
    iprintln,
    peripheral::{itm::Stim, DWT},
};
use embedded_hal::spi::{MODE_0, MODE_1, MODE_2, MODE_3};
// use cortex_m_semihosting::hprintln;
use panic_halt as _;
use rtic::cyccnt::{Instant, U32Ext as _};
use stm32f4xx_hal::{
    // gpio::gpioa::PA0,
    prelude::*,
    // rcc::CFGR,
    spi::Spi,
    stm32,
};

//use crate::hal::gpio::{gpioa::PA0, Edge, Input, PullDown};
//use hal::spi::{Mode, Phase, Polarity};

#[rtic::app(device = stm32f4xx_hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        GPIOA: stm32::GPIOA,
        ITM: stm32::ITM,
    }
    #[init(schedule = [toggle])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        let device = cx.device;

        // Configure led:
        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        // setup clocks
        let rcc = device.RCC.constrain();

        let clocks = rcc.cfgr.freeze();

        let stim = &mut core.ITM.stim[0];

        iprintln!(stim, "pmw3389");

        // iprintln!(stim, "hclk {:?}", clocks.hclk().into::());

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        core.DWT.enable_cycle_counter();

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
        let cs = gpiob.pb4.into_push_pull_output();

        let spi = Spi::spi2(
            device.SPI2,
            (sck, miso, mosi),
            // Mode {
            //     polarity: Polarity::IdleLow,
            //     phase: Phase::CaptureOnFirstTransition,
            // },
            MODE_3,
            stm32f4xx_hal::time::KiloHertz(100).into(),
            clocks,
        );

        iprintln!(stim, "clocks:\n hclk {}", clocks.hclk().0);

        let mut pmw3389 = pmw3389::Pmw3389::new(spi, cs).unwrap();
        let id = pmw3389.product_id().unwrap();
        iprintln!(stim, "id {}", id);

        let id = pmw3389.product_id().unwrap();
        iprintln!(stim, "id {}", id);

        let id = pmw3389
            .read_register(pmw3389::Register::RevisionId)
            .unwrap();
        iprintln!(stim, "rev {}", id);

        let id = pmw3389
            .read_register(pmw3389::Register::ShutterLower)
            .unwrap();
        iprintln!(stim, "lower {}", id);

        let id = pmw3389
            .read_register(pmw3389::Register::ShutterUpper)
            .unwrap();
        iprintln!(stim, "upper {}", id);

        let id = pmw3389.read_register(pmw3389::Register::SROMId).unwrap();
        iprintln!(stim, "sromid {}", id);

        let id = pmw3389
            .read_register(pmw3389::Register::InverseProductID)
            .unwrap();
        iprintln!(stim, "-id {}", id);

        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        // let now = cx.start; // the start time of the system

        // Schedule `toggle` to run 8e6 cycles (clock cycles) in the future
        // cx.schedule.toggle(now + 8_000_000.cycles()).unwrap();

        // pass on late resources
        init::LateResources {
            GPIOA: device.GPIOA,
            ITM: core.ITM,
        }
    }

    #[task(resources = [GPIOA, ITM], schedule = [toggle])]
    fn toggle(cx: toggle::Context) {
        static mut TOGGLE: bool = false;
        iprintln!(&mut cx.resources.ITM.stim[0], "foo  @ {:?}", Instant::now());

        if *TOGGLE {
            cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        *TOGGLE = !*TOGGLE;
        cx.schedule
            .toggle(cx.scheduled + 8_000_000.cycles())
            .unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};

mod pmw3389 {

    use embedded_hal::blocking::spi::{Transfer, Write};
    use embedded_hal::digital::v2::OutputPin;
    use embedded_hal::spi::Mode;

    #[allow(dead_code)]
    #[derive(Clone, Copy)]
    pub enum Register {
        ProductId = 0x00,
        RevisionId = 0x01,
        Motion = 0x02,
        DeltaXL = 0x03,
        DeltaXH = 0x04,
        DeltaYL = 0x05,
        DeltaYH = 0x06,
        SQUAL = 0x07,
        RawDataSum = 0x08,
        MaximumRawdata = 0x09,
        MinimumRawdata = 0x0A,
        ShutterLower = 0x0B,
        ShutterUpper = 0x0C,
        RippleControl = 0x0D,
        ResoultionL = 0x0E,
        ResolutionH = 0x0F,
        Config2 = 0x10,
        AngleTune = 0x11,
        FrameCapture = 0x12,
        SROMEnable = 0x13,
        RunDownshift = 0x14,
        Rest1RateLower = 0x15,
        Rest1RateUpper = 0x16,
        Rest1Downshift = 0x17,
        Rest2RateLower = 0x18,
        Rest2RateUpper = 0x19,
        Rest2Downshift = 0x1A,
        Rest3RateLower = 0x1B,
        Rest3RateUpper = 0x1C,
        Observation = 0x24,
        DataOutLower = 0x25,
        DataOutUpper = 0x26,
        RawDataDump = 0x29,
        SROMId = 0x2A,
        MinSQRun = 0x2B,
        RawDataThreshold = 0x2C,
        Control2 = 0x2D,
        Config5L = 0x2E,
        Config5H = 0x2F,
        PowerUpReset = 0x3A,
        Shutdown = 0x3B,
        InverseProductID = 0x3F,
        LiftCutoffTune3 = 0x41,
        AngleSnap = 0x42,
        LiftCutoffTune1 = 0x4A,
        MotionBurst = 0x50,
        LiftCutoffTune1Timeout = 0x58,
        LiftCutoffTune1MinLength = 0x5A,
        SROMLoadBurst = 0x62,
        LiftConfig = 0x63,
        RawDataBurst = 0x64,
        LiftCutoffTune2 = 0x65,
        LiftCutoffTune2Timeout = 0x71,
        LiftCutoffTune2MinLength = 0x72,
        PWMPeriodCnt = 0x73,
        PWMWidthCnt = 0x74,
    }

    impl Register {
        fn addr(self) -> u8 {
            self as u8
        }
    }

    const READ: u8 = 1 << 7;
    const WRITE: u8 = 0 << 7;
    const MULTI: u8 = 1 << 6;
    const SINGLE: u8 = 0 << 6;

    pub struct Pmw3389<SPI, CS> {
        spi: SPI,
        cs: CS,
    }

    impl<SPI, CS, E> Pmw3389<SPI, CS>
    where
        SPI: Transfer<u8, Error = E> + Write<u8, Error = E>,
        CS: OutputPin,
    {
        /// Creates a new driver from a SPI peripheral and a NCS pin
        pub fn new(spi: SPI, cs: CS) -> Result<Self, E> {
            let mut pmw3389 = Pmw3389 { spi, cs };

            // power up and enable all the axes
            // l3gd20.write_register(Register::CTRL_REG1, 0b00_00_1_111)?;

            Ok(pmw3389)
        }

        /// Reads the ProductId register; should return `0x47`
        pub fn product_id(&mut self) -> Result<u8, E> {
            self.read_register(Register::ProductId)
        }

        // /// Temperature measurement + gyroscope measurements
        // pub fn all(&mut self) -> Result<Measurements, E> {
        //     let mut bytes = [0u8; 9];
        //     self.read_many(Register::OUT_TEMP, &mut bytes)?;

        //     Ok(Measurements {
        //         gyro: I16x3 {
        //             x: (bytes[3] as u16 + ((bytes[4] as u16) << 8)) as i16,
        //             y: (bytes[5] as u16 + ((bytes[6] as u16) << 8)) as i16,
        //             z: (bytes[7] as u16 + ((bytes[8] as u16) << 8)) as i16,
        //         },
        //         temp: bytes[1] as i8,
        //     })
        // }

        // /// Gyroscope measurements
        // pub fn gyro(&mut self) -> Result<I16x3, E> {
        //     let mut bytes = [0u8; 7];
        //     self.read_many(Register::OUT_X_L, &mut bytes)?;

        //     Ok(I16x3 {
        //         x: (bytes[1] as u16 + ((bytes[2] as u16) << 8)) as i16,
        //         y: (bytes[3] as u16 + ((bytes[4] as u16) << 8)) as i16,
        //         z: (bytes[5] as u16 + ((bytes[6] as u16) << 8)) as i16,
        //     })
        // }

        // /// Temperature sensor measurement
        // pub fn temp(&mut self) -> Result<i8, E> {
        //     Ok(self.read_register(Register::OUT_TEMP)? as i8)
        // }

        // /// Read `STATUS_REG` of sensor
        // pub fn status(&mut self) -> Result<Status, E> {
        //     let sts = self.read_register(Register::STATUS_REG)?;
        //     Ok(Status::from_u8(sts))
        // }

        // /// Get the current Output Data Rate
        // pub fn odr(&mut self) -> Result<Odr, E> {
        //     // Read control register
        //     let reg1 = self.read_register(Register::CTRL_REG1)?;
        //     Ok(Odr::from_u8(reg1))
        // }

        // /// Set the Output Data Rate
        // pub fn set_odr(&mut self, odr: Odr) -> Result<&mut Self, E> {
        //     self.change_config(Register::CTRL_REG1, odr)
        // }

        // /// Get current Bandwidth
        // pub fn bandwidth(&mut self) -> Result<Bandwidth, E> {
        //     let reg1 = self.read_register(Register::CTRL_REG1)?;
        //     Ok(Bandwidth::from_u8(reg1))
        // }

        // /// Set low-pass cut-off frequency (i.e. bandwidth)
        // ///
        // /// See `Bandwidth` for further explanation
        // pub fn set_bandwidth(&mut self, bw: Bandwidth) -> Result<&mut Self, E> {
        //     self.change_config(Register::CTRL_REG1, bw)
        // }

        // /// Get the current Full Scale Selection
        // ///
        // /// This is the sensitivity of the sensor, see `Scale` for more information
        // pub fn scale(&mut self) -> Result<Scale, E> {
        //     let scl = self.read_register(Register::CTRL_REG4)?;
        //     Ok(Scale::from_u8(scl))
        // }

        // /// Set the Full Scale Selection
        // ///
        // /// This sets the sensitivity of the sensor, see `Scale` for more
        // /// information
        // pub fn set_scale(&mut self, scale: Scale) -> Result<&mut Self, E> {
        //     self.change_config(Register::CTRL_REG4, scale)
        // }

        pub fn read_register(&mut self, reg: Register) -> Result<u8, E> {
            let _ = self.cs.set_low();

            let mut buffer = [reg.addr() & 0x7f, 0];
            self.spi.transfer(&mut buffer)?;

            let _ = self.cs.set_high();

            Ok(buffer[1])
        }

        /// Read multiple bytes starting from the `start_reg` register.
        /// This function will attempt to fill the provided buffer.
        fn read_many(&mut self, start_reg: Register, buffer: &mut [u8]) -> Result<(), E> {
            let _ = self.cs.set_low();
            buffer[0] = start_reg.addr() | MULTI | READ;
            self.spi.transfer(buffer)?;
            let _ = self.cs.set_high();

            Ok(())
        }

        fn write_register(&mut self, reg: Register, byte: u8) -> Result<(), E> {
            let _ = self.cs.set_low();

            let buffer = [reg.addr() | SINGLE | WRITE, byte];
            self.spi.write(&buffer)?;

            let _ = self.cs.set_high();

            Ok(())
        }

        // /// Change configuration in register
        // ///
        // /// Helper function to update a particular part of a register without
        // /// affecting other parts of the register that might contain desired
        // /// configuration. This allows the `L3gd20` struct to be used like
        // /// a builder interface when configuring specific parameters.
        // fn change_config<B: BitValue>(&mut self, reg: Register, bits: B) -> Result<&mut Self, E> {
        //     // Create bit mask from width and shift of value
        //     let mask = B::mask() << B::shift();
        //     // Extract the value as u8
        //     let bits = (bits.value() << B::shift()) & mask;
        //     // Read current value of register
        //     let current = self.read_register(reg)?;
        //     // Use supplied mask so we don't affect more than necessary
        //     let masked = current & !mask;
        //     // Use `or` to apply the new value without affecting other parts
        //     let new_reg = masked | bits;
        //     self.write_register(reg, new_reg)?;
        //     Ok(self)
        // }
    }
}
