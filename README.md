# App

## Resources

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

## Connections

- [USB Cable]

| Signl | Color | Pin  |
| ----- | ----- | ---- |
| V+    | Red   | ---- |
| D-    | White | PA11 |
| D+    | Green | PA12 |
| Gnd   | Black | ---- |

D+ used for re-enumeration

## Debug interface

- Serial Wire debugging uses pins PA13 and PA14. So refrain from using those unless absolutely necessary.

## Troubleshooting

### Fail to connect with openocd

First check that your stilnk nucleo programmer is found by the host.

```shell
> lsusb
...
Bus 003 Device 013: ID 0483:374b STMicroelectronics ST-LINK/V2.1
...
```

If not check your USB cable. Notice, you need a USB data cable (not a USB charging cable).
If the problem is still there, there might be a USB issue with the host (or VM if you run Linux under a VM that is).

If you get a connection error similar to the below:

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

First thing to try is holding the reset button while connecting.

If this does not work you can try to erase the flash memory (the program running on the STM32F401/F11).

``` shell
> st-util erase
st-flash 1.6.1
2021-01-11T16:02:14 INFO common.c: F4xx (Dynamic Efficency): 96 KiB SRAM, 512 KiB flash in at least 16 KiB pages.
Mass erasing.......
```

If this still does not work you can connect `Boot0` to `VDD` (found on CN7 pins 7, and 5 respectively). Unplug/replug the Nucleo and try to erase the flash as above.

If this still does not work, the Nucleo might actually been damaged, or that the problem is the usb-cable or host machine related.

