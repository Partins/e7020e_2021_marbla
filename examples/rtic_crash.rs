#![no_main]
#![no_std]

use core::ptr;
use panic_halt as _;
use stm32f4;

#[rtic::app(device = stm32f4)]
const APP: () = {
    #[init]
    fn init(_cx: init::Context) {
        unsafe {
            // read an address outside of the RAM region; this causes a HardFault exception
            ptr::read_volatile(0x2FFF_FFFF as *const u32);
        }
    }
};

// Here you can inspect the call stack (found to the left in vscode).
//
// The default implementation for `HardFault` exception is just an infinite loop.
// Press the pause symbol to halt the processor:
//
// The upmost item in CALL STACK, is the current frame:
// (the infinite loop loop)
//
// The bottom most item is the start of the program (the generated main).
//
// In between, you can see the calls made
// main->init->read_volatile->HardFault->compiler_fence
//
// Click on init, and you will see that line 14 in this application caused the
// erroneous read operation.
