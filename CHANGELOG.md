# Changelog

## 2021-03-07

- examples/rtic_bare7.rs, using embedded HAL.
- examples/rtic_bare8.rs, serial communication, bad design.
- examples/rtic_bare9.rs, serial communication, good design.
  
## 2021-03-05

- examples/rtic_bare6.rs, setup and validate the clock tree.

## 2021-02-28

- examples/rtic_bare2.rs, raw timer access.
- examples/rtic_bare3.rs, timing abstractions.
- examples/rtic_bare4.rs, a simple bare metal peripheral access API.
- examples/rtic_bare5.rs, write your own C-like peripheral access API.

## 2021-02-26

- examples/bare1.rs, bare metal 101!
  
## 2021-02-23

- examples/rtic_blinky.rs, added instructions to terminal based debugging
  
## 2021-02-22

- memory.x, reduced flash size to 128k to match light-weight target
- Cargo.toml, updated dependencies to latest stm32f4xx-hal/pac

Some experiments (wip):

- examples/rtt_rtic_i2c.rs, spi emulation over i2c
- src/pwm3389e, driver using emulated spi

## 2021-02-16

- rtt_rtic_usb_mouse updated
  Notice, requires release build

## 2021-02-15

- Initial release for the e7020e course 2021
  