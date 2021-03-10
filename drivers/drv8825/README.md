# DRV8825 Driver [![crates.io](https://img.shields.io/crates/v/drv8825.svg)](https://crates.io/crates/drv8825) [![Documentation](https://docs.rs/drv8825/badge.svg)](https://docs.rs/drv8825) ![CI Build](https://github.com/flott-motion/stepper/workflows/CI%20Build/badge.svg)

## About

Rust driver crate for the [DRV8825] stepper motor driver. Carrier boards for this chip are [available from Pololu].

This crate is a specialized facade for the [Stepper] library. Please consider using Stepper directly, as it provides drivers for more stepper motor drivers, as well as an interface to abstract over them.

See [Stepper] for more documentation and usage examples.

## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.

[drv8825]: https://www.ti.com/product/DRV8825
[available from pololu]: https://www.pololu.com/category/154/
[Stepper]: https://crates.io/crates/stepper
[zero clause bsd license]: https://opensource.org/licenses/0BSD
[license.md]: https://github.com/flott-motion/stepper/blob/main/LICENSE.md
