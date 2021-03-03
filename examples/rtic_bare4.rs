//! rtic_bare4.rs
//!
//! Access to Peripherals
//!
//! What it covers:
//! - Raw pointers
//! - Volatile read/write
//! - Busses and clocking
//! - GPIO (a primitive abstraction)

#![no_std]
#![no_main]

extern crate cortex_m;
extern crate panic_halt;
use stm32f4;

// Peripheral addresses as constants
#[rustfmt::skip]
mod address {
    pub const PERIPH_BASE: u32      = 0x40000000;
    pub const AHB1PERIPH_BASE: u32  = PERIPH_BASE + 0x00020000;
    pub const RCC_BASE: u32         = AHB1PERIPH_BASE + 0x3800;
    pub const RCC_AHB1ENR: u32      = RCC_BASE + 0x30;
    pub const GBPIA_BASE: u32       = AHB1PERIPH_BASE + 0x0000;
    pub const GPIOA_MODER: u32      = GBPIA_BASE + 0x00;
    pub const GPIOA_BSRR: u32       = GBPIA_BASE + 0x18;
}

use address::*;

// see the Reference Manual RM0368 (www.st.com/resource/en/reference_manual/dm00096844.pdf)
// rcc,     chapter 6
// gpio,    chapter 8

#[inline(always)]
fn read_u32(addr: u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const _) }
    // core::ptr::read_volatile(addr as *const _)
}

#[inline(always)]
fn write_u32(addr: u32, val: u32) {
    unsafe {
        core::ptr::write_volatile(addr as *mut _, val);
    }
}

fn wait(i: u32) {
    for _ in 0..i {
        cortex_m::asm::nop(); // no operation (cannot be optimized out)
    }
}

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(_cx: init::Context) {
        // power on GPIOA
        let r = read_u32(RCC_AHB1ENR); // read
        write_u32(RCC_AHB1ENR, r | 1); // set enable

        // configure PA5 as output
        let r = read_u32(GPIOA_MODER) & !(0b11 << (5 * 2)); // read and mask
        write_u32(GPIOA_MODER, r | 0b01 << (5 * 2)); // set output mode

        // and alter the data output through the BSRR register
        // this is more efficient as the read register is not needed.

        loop {
            // set PA5 high
            write_u32(GPIOA_BSRR, 1 << 5); // set bit, output hight (turn on led)
            wait(10_000);

            // set PA5 low
            write_u32(GPIOA_BSRR, 1 << (5 + 16)); // clear bit, output low (turn off led)
            wait(10_000);
        }
    }
};

// 0.  Build and run the application (Cortex Debug).
//
// 1.  Did you enjoy the blinking?
//
//    ** your answer here **
//
//    Now lookup the data-sheets, and read each section referred,
//    6.3.11, 8.4.1, 8.4.7
//
//    Document each low level access *code* by the appropriate section in the
//    data sheet.
//
//    Commit your answers (bare4_1)
//
// 2. Comment out line 38 and uncomment line 39 (essentially omitting the `unsafe`)
//
//    //unsafe { core::ptr::read_volatile(addr as *const _) }
//    core::ptr::read_volatile(addr as *const _)
//
//    What was the error message and explain why.
//
//    ** your answer here **
//
//    Digging a bit deeper, why do you think `read_volatile` is declared `unsafe`.
//    (https://doc.rust-lang.org/core/ptr/fn.read_volatile.html, for some food for thought )
//
//    ** your answer here **
//
//    Commit your answers (bare4_2)
//
// 3. Volatile read/writes are explicit *volatile operations* in Rust, while in C they
//    are declared at type level (i.e., access to variables declared volatile amounts to
//    volatile reads/and writes).
//
//    Both C and Rust (even more) allows code optimization to re-order operations, as long
//    as data dependencies are preserved.
//
//    Why is it important that ordering of volatile operations are ensured by the compiler?
//
//    ** your answer here **
//
//    Give an example in the above code, where reordering might make things go horribly wrong
//    (hint, accessing a peripheral not being powered...)
//
//    ** your answer here **
//
//    Without the non-reordering property of `write_volatile/read_volatile` could that happen in theory
//    (argue from the point of data dependencies).
//
//    ** your answer here **
//
//    Commit your answers (bare4_3)
