#![no_std]

pub mod pmw3389;
pub mod pmw3389e;

use stm32f4xx_hal::{prelude::*, rcc::Clocks, stm32};

pub struct DwtDelay {
    clocks: Clocks,
}

impl DwtDelay {
    pub fn new(dwt: &mut stm32::DWT, clocks: Clocks) -> DwtDelay {
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        stm32::DWT::unlock();
        dwt.enable_cycle_counter();
        Self { clocks }
    }
}

impl _embedded_hal_blocking_delay_DelayUs<u32> for DwtDelay {
    fn delay_us(&mut self, us: u32) {
        let freq_m_hertz = self.clocks.hclk().0 / 1_000_000;

        let start = stm32::DWT::get_cycle_count() as i32;
        let end = start.wrapping_add((us * freq_m_hertz) as i32);

        while (stm32::DWT::get_cycle_count() as i32).wrapping_sub(end) < 0 {
            // this nop should be ok as the `DWT::get_cycle_count() provides side effects
        }
    }
}

impl _embedded_hal_blocking_delay_DelayMs<u32> for DwtDelay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1000)
    }
}
