<h1 align="center">
AUTD3: Airborne Ultrasound Tactile Display 3
</h1>

<div align="center">

![build](https://github.com/shinolab/autd3/workflows/build/badge.svg)
[![codecov](https://codecov.io/gh/shinolab/autd3-rs/graph/badge.svg)](https://codecov.io/gh/shinolab/autd3-rs)
[![Crate.io version](https://img.shields.io/crates/v/autd3)](https://crates.io/crates/autd3)

</div>

<p align="center">
Airborne Ultrasound Tactile Display (AUTD) is a midair haptic device that can remotely produce tactile sensation on a human skin surface without wearing devices.
Please see <a href="https://hapislab.org/en/airborne-ultrasound-tactile-display">our laboratory homepage</a> for more details on AUTD.
This repository contains a client library to drive AUTD version 3 devices.
This cross-platform library supports Windows, macOS, and Linux (including Single Board Computer such as Raspberry Pi).
</p>

> [!WARNING]  
> From v17.0.0, the software is completely incompatible with v2.x and v3.x firmware.
> Before using this library, write the latest firmware in `firmware`. For more information, please see [autd3-firmware/README](https://github.com/shinolab/autd3-firmware).

## Example

* See [examples](./examples)

    * If you are using Linux/macOS, you may need to run as root.

## Citing

* If you use this SDK in your research, please consider including the following citation in your publications:

   * [S. Suzuki, S. Inoue, M. Fujiwara, Y. Makino, and H. Shinoda, "AUTD3: Scalable Airborne Ultrasound Tactile Display," in IEEE Transactions on Haptics, DOI: 10.1109/TOH.2021.3069976.](https://ieeexplore.ieee.org/document/9392322)
   * S. Inoue, Y. Makino and H. Shinoda "Scalable Architecture for Airborne Ultrasound Tactile Display," Asia Haptics 2016

## LICENSE

* See [LICENSE](./LICENSE)

# Author

Shun Suzuki, 2022-2023
