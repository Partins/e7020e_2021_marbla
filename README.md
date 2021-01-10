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

- Serial Wire debugging uses pins PA13 and PA14.
  