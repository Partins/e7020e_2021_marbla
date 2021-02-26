# RTIC on the STM32F4xx Nucleo board

All tooling have been developed and tested under Linux. Any modern Linux distro should work, we usually recommend Arch linux as it provides a great package manager with rolling releases. If you want to run Arch, but don't want to install everything from scratch, you may opt for [Manjaro](https://manjaro.org/) or [Endeavour](https://endeavouros.com/). You will get the best user experience by a native install, but you may run Linux under a VM like virtualbox, or vmware (the player is free). You should install the guest extensions, to get better graphics performance (and perhaps better USB forwarding). Since you will connect your Nucleo using USB, you must make sure that USB port forwarding works (the Nucleo stlink programmer is a USB2 device running in full speed 12MBit).

This repo will be updated with more information throughout the course so please check the `CHANGELOG.md` and recent commits to see what has changed. (You should `pull` the upstream to keep your repository updated.) If you have suggestions to further improvements, please raise an issue and/or create a merge/pull request.

## Rust

We assume Rust to be installed using [rustup](https://www.rust-lang.org/tools/install).

Additionally you need to install the `thumbv7em-none-eabi` target.

```shell
> rustup target add thumbv7em-none-eabi 
```

You also need [cargo-binutils](https://github.com/rust-embedded/cargo-binutils), for inspecting the generated binaries. You install Rust applications through `cargo`

```shell
> cargo install cargo-binutils
```

There are a number of other useful [cargo subcommands](https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands), notably `cargo-bloat` (that gives you info on the size of different sections of the generated binary), `cargo-tree` (that list your dependency tree), etc.

## For RTT tracing

We assume the following tools are in place:

- [probe-run](https://crates.io/crates/probe-run)

## For programming and low level `gdb` based debugging

Linux tooling:

- `stlink`, this package will install programming utilities like `st-flash` (useful if you need to recover a bricked target by erasing the flash), and setup `udev` rules, allowing you to access the USB device without `sudo`. Install may require you to login/logout to have new `udev` rules applied.
- `openocd`, this tool allows the host to connect to the (stlink) programmer.
- `arm-none-eabi-gdb`, or `gdb-multiarch` (dependent on Linux distro). This tool allows you to program (flash) and debug your target.

## Editor

You may use any editor of choice. `vscode` supports Rust using the  `rust-analyzer` plugin. You may also want to install the `Cortex Debug` plugin. In the `.vscode` folder, there are a number of configuration files (`launch.json` for target debugging, `tasks.json` for building, etc.).

## Useful Resources

- Nucleo 64
  - [UM1724 - stm32 Nucleo-64](https://www.st.com/resource/en/user_manual/dm00105823-stm32-nucleo64-boards-mb1136-stmicroelectronics.pdf).
  - [Nucleo 64 Schematics](https://www.st.com/resource/en/schematic_pack/nucleo_64pins_sch.zip) (The file MB1136.pdf is the schematics in pdf.)
  - [stm32f4xx_hal](https://docs.rs/stm32f4xx-hal/0.8.3/stm32f4xx_hal/) documentation of the HAL API, and [git repository](https://github.com/stm32-rs/stm32f4xx-hal).

- STM32F01/FO11
  - [RM0383 - F411 Reference Manual](https://www.st.com/resource/zh/reference_manual/dm00119316-stm32f411xce-advanced-armbased-32bit-mcus-stmicroelectronics.pdf) 
  - [RM0368 - F401 Reference Manual](https://www.st.com/resource/en/reference_manual/dm00096844-stm32f401xbc-and-stm32f401xde-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
  - [PM0214 - M4 Programming manual](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=&ved=2ahUKEwjOtd645OTtAhXEHXcKHdwYCoQQFjAAegQIBhAC&url=https%3A%2F%2Fwww.st.com%2Fresource%2Fen%2Fprogramming_manual%2Fdm00046982-stm32-cortex-m4-mcus-and-mpus-programming-manual-stmicroelectronics.pdf&usg=AOvVaw0n3XXybtMMDbifhDZse1Pl)

- PixArt PMW33xx Optical Navigation Chip
  - [PMW3389DM-T3QU](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=&ved=2ahUKEwicx5OA9eTtAhWC-yoKHVfKAJ0QFjAAegQIBhAC&url=https%3A%2F%2Fwww.pixart.com%2F_getfs.php%3Ftb%3Dproduct%26id%3D4%26fs%3Dck2_fs_cn&usg=AOvVaw1A1rR533Pt-7EgnVSS-_ch), optical navigation chip
  - [Jack Enterprise Breakout Board](https://www.tindie.com/products/jkicklighter/pmw3389-motion-sensor/), an example design with software linked.

- General Embedded
  - [Introduction to SPI](https://www.analog.com/en/analog-dialogue/articles/introduction-to-spi-interface.html#), a short introduction to the SPI interface.

---

## Examples

### VSCODE based debug and trace

Some simple bare metal examples for you to try out before starting to run your own code:
Using `vscode` just press F5 to launch and debug the program in the currently active vscode window.

- `rtic_hello.rs`, this example uses semihosting to print the output terminal. Open the `OUTPUT` pane, and select `Adapter Output` (which is the openocd console).
- `itm_rtic_hello.rs`, this examples uses the ITM trace to print to an output trace channel. Open the `OUTPUT` pane, and select `SWO:ITM[port:0, type:console]`.
- `rtic_panic.rs`, this example shows how to trace panic messages (in this case over semihosting).  Open the `OUTPUT` pane, and select `Adapter Output` (which is the openocd console).
- `rtic_crash.rs`, this example shows how to trace a HardFault (an error raised by the ARM processor).
  
---

### Exercises

Bare metal programming:

- `bare1.rs`, in this exercise you learn about debugging, inspecting the generated assembly code, inline assembly, and about checked vs. unchecked (wrapping) arithmetics. Provides essential skills and understanding of low level (bare metal) programming.

---

### Console based debug and trace

- `rtt_rtic_hello.rs`, this example uses the RTT framework for tracing.

  ```shell
  > cargo run --example rtt_rtic_hello
  ```

---

## Nucleo Connections

---

Some of the examples need external connection to the Nucleo to work.

---

### USB example

| Signal | Color | Pin  | Nucleo    |
| ------ | ----- | ---- | --------- |
| V+     | Red   |      |           |
| D-     | White | PA11 | CN10 - 14 |
| D+     | Green | PA12 | CN10 - 12 |
| Gnd    | Black |      | CN10 - 9  |

D+ used for re-enumeration. You don't need to connect the V+ from the USB cable, as the NUCLEO is self powered.

---

### PWM example

| Signal | Pin | Nucleo  |
| ------ | --- | ------- |
| PWM1   | PA8 | CN9 - 8 |
| PWM2   | PA9 | CN5 - 1 |

---

### I2C example

| Signal   | Pin | Nucleo |
| -------- | --- | ------ |
| I2C1_SDA | PB9 | CN10-5 |
| I2C1_SCL | PB8 | CN10-3 |
| +3.3v    |     | CN7-16 |
| GND      |     | Gnd    |

## Debug interface

- Serial Wire debugging uses pins PA13 and PA14. So refrain from using those unless absolutely necessary.

---


## Troubleshooting

---

### Fail to connect or program (flash) your target

- Make sure you have the latest version of the [stlink](https://www.st.com/en/development-tools/stsw-link007.html) firmware (2.37.27 or later).

- Check that your stlink nucleo programmer is found by the host.

  ```shell
  > lsusb
  ...
  Bus 003 Device 013: ID 0483:374b STMicroelectronics ST-LINK/V2.1
  ...
  ```

  If not check your USB cable. Notice, you need a USB data cable (not a USB charging cable).
  If the problem is still there, there might be a USB issue with the host (or VM if you run Linux under a VM that is).

- If you get a connection error similar to the below:

  ```shell
  > openocd -f openocd.cfg
  Open On-Chip Debugger 0.10.0+dev-01157-gd6541a811-dirty (2020-03-28-18:34)
  Licensed under GNU GPL v2
  For bug reports, read
        http://openocd.org/doc/doxygen/bugs.html
  Info : auto-selecting first available session transport "hla_swd". To override use 'transport select <transport>'.
  Info : The selected transport took over low-level target control. The results might differ compared to plain JTAG/SWD
  Info : Listening on port 6666 for tcl connections
  Info : Listening on port 4444 for telnet connections
  Info : clock speed 2000 kHz
  Info : STLINK V2J37M26 (API v2) VID:PID 0483:374B
  Info : Target voltage: 3.243627
  Info : stm32f4x.cpu: hardware has 6 breakpoints, 4 watchpoints
  Info : Listening on port 3333 for gdb connections
  Error: jtag status contains invalid mode value - communication failure
  Polling target stm32f4x.cpu failed, trying to reexamine
  Examination failed, GDB will be halted. Polling again in 100ms
  Info : Previous state query failed, trying to reconnect
  Error: jtag status contains invalid mode value - communication failure
  Polling target stm32f4x.cpu failed, trying to reexamine 
  ```

  - First thing to try is holding the reset button while connecting.

  - If this does not work you can try to erase the flash memory (the program running on the STM32F401/F11).

    ``` shell
    > st-util erase
    st-flash 1.6.1
    2021-01-11T16:02:14 INFO common.c: F4xx (Dynamic Efficency): 96 KiB SRAM, 512 KiB flash in at least 16 KiB pages.
    Mass erasing.......
    ```

  - If this still does not work you can connect `Boot0` to `VDD` (found on CN7 pins 7, and 5 respectively). Unplug/replug   the Nucleo and try to erase the flash as above.
  
  - If this still does not work, the Nucleo might actually been damaged, or that the problem is the usb-cable or host   machine related.
