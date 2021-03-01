//! rtic_bare5.rs
//!
//! C Like Peripheral API
//!
//! What it covers:
//! - abstractions in Rust
//! - structs and implementations

#![no_std]
#![no_main]

extern crate cortex_m;
extern crate panic_semihosting;

// C like API...
mod stm32f40x {
    #[allow(dead_code)]
    use core::{cell, ptr};

    #[rustfmt::skip]
    mod address {
        pub const PERIPH_BASE: u32      = 0x40000000;
        pub const AHB1PERIPH_BASE: u32  = PERIPH_BASE + 0x00020000;
        pub const RCC_BASE: u32         = AHB1PERIPH_BASE + 0x3800;
        pub const GPIOA_BASE: u32       = AHB1PERIPH_BASE + 0x0000;
    }
    use address::*;

    pub struct VolatileCell<T> {
        pub value: cell::UnsafeCell<T>,
    }

    impl<T> VolatileCell<T> {
        #[inline(always)]
        pub fn read(&self) -> T
        where
            T: Copy,
        {
            unsafe { ptr::read_volatile(self.value.get()) }
        }

        #[inline(always)]
        pub fn write(&self, value: T)
        where
            T: Copy,
        {
            unsafe { ptr::write_volatile(self.value.get(), value) }
        }
    }

    // modify (reads, modifies a field, and writes the volatile cell)
    //
    // parameters:
    // offset (field offset)
    // width  (field width)
    // value  (new value that the field should take)
    //
    impl VolatileCell<u32> {
        #[inline(always)]
        pub fn modify(&self, offset: u8, width: u8, value: u32) {
            // your code here
        }
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    #[rustfmt::skip]
    pub struct RCC {
        pub CR:         VolatileCell<u32>,      // < RCC clock control register,                                    Address offset: 0x00 
        pub PLLCFGR:    VolatileCell<u32>,      // < RCC PLL configuration register,                                Address offset: 0x04 
        pub CFGR:       VolatileCell<u32>,      // < RCC clock configuration register,                              Address offset: 0x08 
        pub CIR:        VolatileCell<u32>,      // < RCC clock interrupt register,                                  Address offset: 0x0C 
        pub AHB1RSTR:   VolatileCell<u32>,      // < RCC AHB1 peripheral reset register,                            Address offset: 0x10 
        pub AHB2RSTR:   VolatileCell<u32>,      // < RCC AHB2 peripheral reset register,                            Address offset: 0x14 
        pub AHB3RSTR:   VolatileCell<u32>,      // < RCC AHB3 peripheral reset register,                            Address offset: 0x18 
        pub RESERVED0:  VolatileCell<u32>,      // < Reserved, 0x1C                                                                      
        pub APB1RSTR:   VolatileCell<u32>,      // < RCC APB1 peripheral reset register,                            Address offset: 0x20 
        pub APB2RSTR:   VolatileCell<u32>,      // < RCC APB2 peripheral reset register,                            Address offset: 0x24 
        pub RESERVED1:  [VolatileCell<u32>; 2], // < Reserved, 0x28-0x2C                                                                 
        pub AHB1ENR:    VolatileCell<u32>,      // < RCC AHB1 peripheral clock register,                            Address offset: 0x30 
        pub AHB2ENR:    VolatileCell<u32>,      // < RCC AHB2 peripheral clock register,                            Address offset: 0x34 
        pub AHB3ENR:    VolatileCell<u32>,      // < RCC AHB3 peripheral clock register,                            Address offset: 0x38 
        pub RESERVED2:  VolatileCell<u32>,      // < Reserved, 0x3C                                                                      
        pub APB1ENR:    VolatileCell<u32>,      // < RCC APB1 peripheral clock enable register,                     Address offset: 0x40 
        pub APB2ENR:    VolatileCell<u32>,      // < RCC APB2 peripheral clock enable register,                     Address offset: 0x44 
        pub RESERVED3:  [VolatileCell<u32>; 2], // < Reserved, 0x48-0x4C                                                                 
        pub AHB1LPENR:  VolatileCell<u32>,      // < RCC AHB1 peripheral clock enable in low power mode register,   Address offset: 0x50 
        pub AHB2LPENR:  VolatileCell<u32>,      // < RCC AHB2 peripheral clock enable in low power mode register,   Address offset: 0x54 
        pub AHB3LPENR:  VolatileCell<u32>,      // < RCC AHB3 peripheral clock enable in low power mode register,   Address offset: 0x58 
        pub RESERVED4:  VolatileCell<u32>,      // < Reserved, 0x5C                                                                      
        pub APB1LPENR:  VolatileCell<u32>,      // < RCC APB1 peripheral clock enable in low power mode register,   Address offset: 0x60 
        pub APB2LPENR:  VolatileCell<u32>,      // < RCC APB2 peripheral clock enable in low power mode register,   Address offset: 0x64 
        pub RESERVED5:  [VolatileCell<u32>; 2], // < Reserved, 0x68-0x6C                                                                 
        pub BDCR:       VolatileCell<u32>,      // < RCC Backup domain control register,                            Address offset: 0x70 
        pub CSR:        VolatileCell<u32>,      // < RCC clock control & status register,                           Address offset: 0x74 
        pub RESERVED6:  [VolatileCell<u32>; 2], // < Reserved, 0x78-0x7C                                                                 
        pub SSCGR:      VolatileCell<u32>,      // < RCC spread spectrum clock generation register,                 Address offset: 0x80 
        pub PLLI2SCFGR: VolatileCell<u32>,      // < RCC PLLI2S configuration register,                             Address offset: 0x84 
    }

    impl RCC {
        pub fn get() -> *mut RCC {
            address::RCC_BASE as *mut RCC
        }
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    #[rustfmt::skip]
    pub struct GPIOA {
        pub MODER:      VolatileCell<u32>,      // < GPIO port mode register,                                       Address offset: 0x00     
        pub OTYPER:     VolatileCell<u32>,      // < GPIO port output type register,                                Address offset: 0x04     
        pub OSPEEDR:    VolatileCell<u32>,      // < GPIO port output speed register,                               Address offset: 0x08     
        pub PUPDR:      VolatileCell<u32>,      // < GPIO port pull-up/pull-down register,                          Address offset: 0x0C     
        pub IDR:        VolatileCell<u32>,      // < GPIO port input data register,                                 Address offset: 0x10     
        pub ODR:        VolatileCell<u32>,      // < GPIO port output data register,                                Address offset: 0x14     
        pub BSRRL:      VolatileCell<u16>,      // < GPIO port bit set/reset low register,                          Address offset: 0x18     
        pub BSRRH:      VolatileCell<u16>,      // < GPIO port bit set/reset high register,                         Address offset: 0x1A     
        pub LCKR:       VolatileCell<u32>,      // < GPIO port configuration lock register,                         Address offset: 0x1C     
        pub AFR:        [VolatileCell<u32>;2],  // < GPIO alternate function registers,                             Address offset: 0x20-0x24
    }

    impl GPIOA {
        pub fn get() -> *mut GPIOA {
            GPIOA_BASE as *mut GPIOA
        }
    }
}
use stm32f40x::*;

// see the Reference Manual RM0368 (www.st.com/resource/en/reference_manual/dm00096844.pdf)
// rcc,     chapter 6
// gpio,    chapter 8

fn wait(i: u32) {
    for _ in 0..i {
        cortex_m::asm::nop(); // no operation (cannot be optimized out)
    }
}

// simple test of Your `modify`
//
fn test_modify() {
    let t: VolatileCell<u32> = VolatileCell {
        value: core::cell::UnsafeCell::new(0),
    };
    t.write(0);
    assert!(t.read() == 0);
    t.modify(3, 3, 0b10101);
    //     10101
    //    ..0111000
    //    ---------
    //    000101000
    assert!(t.read() == 0b101 << 3);
    t.modify(4, 3, 0b10001);
    //    000101000
    //      111
    //      001
    //    000011000
    assert!(t.read() == 0b011 << 3);
    //
    // add more tests here if you like
}

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(_cx: init::Context) {
        let rcc = unsafe { &mut *RCC::get() }; // get the reference to RCC in memory
        let gpioa = unsafe { &mut *GPIOA::get() }; // get the reference to GPIOA in memory

        // power on GPIOA
        let r = rcc.AHB1ENR.read(); // read
        rcc.AHB1ENR.write(r | 1 << (0)); // set enable

        // configure PA5 as output
        let r = gpioa.MODER.read() & !(0b11 << (5 * 2)); // read and mask
        gpioa.MODER.write(r | 0b01 << (5 * 2)); // set output mode

        // test_modify();

        loop {
            // set PA5 high
            gpioa.BSRRH.write(1 << 5); // set bit, output hight (turn on led)

            // alternatively to set the bit high we can
            // read the value, or with PA5 (bit 5) and write back
            // gpioa.ODR.write(gpioa.ODR.read() | (1 << 5));

            wait(10_000);

            // set PA5 low
            gpioa.BSRRL.write(1 << 5); // clear bit, output low (turn off led)

            // alternatively to clear the bit we can
            // read the value, mask out PA5 (bit 5) and write back
            // gpioa.ODR.write(gpioa.ODR.read() & !(1 << 5));
            wait(10_000);
        }
    }
};

// 1. C like API.
//    Using C the .h files are used for defining interfaces, like function signatures (prototypes),
//    structs and macros (but usually not the functions themselves).
//
//    Here is a peripheral abstraction quite similar to what you would find in the .h files
//    provided by ST (and other companies). Actually, the file presented here is mostly a
//    cut/paste/replace of the stm32f40x.h, just Rustified.
//
//    In the loop we access PA5 through bit set/clear operations.
//    Comment out those operations and uncomment the ODR based accesses.
//    (They should have the same behavior, but is a bit less efficient.)
//
//    Run and see that the program behaves the same.
//
//    Commit your answers (bare5_1)
//
// 2. Extend the read/write API with a `modify` for u32, taking the
//    - address (&mut u32),
//    - field offset (in bits, u8),
//    - field width (in bits, u8),
//    - and value (u32).
//
//    Implement and check that running `test` gives you expected behavior.
//
//    Change the code into using your new `modify` API.
//
//    Run and see that the program behaves the same.
//
//    Discussion:
//    As with arithmetic operations, default semantics differ in between
//    debug/dev and release builds.
//    In debug << rhs is checked, rhs must be less than 32 (for 32 bit datatypes).
//
//    Notice, over-shifting (where bits are spilled) is always considered legal,
//    its just the shift amount that is checked.
//    There are explicit unchecked versions available if so wanted.
//
//    We are now approaching a more "safe" to use API.
//    What if we could automatically generate that from Vendors specifications (SVD files)?
//    Wouldn't that be great?
//
//    ** your answer here **
//
//    Commit your answers (bare5_2)
