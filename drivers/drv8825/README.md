# DRV8825 Driver [![crates.io](https://img.shields.io/crates/v/drv8825.svg)](https://crates.io/crates/drv8825) [![Documentation](https://docs.rs/drv8825/badge.svg)](https://docs.rs/drv8825) ![CI Build](https://github.com/braun-embedded/step-dir/workflows/CI%20Build/badge.svg)

## About

Rust driver crate for the [DRV8825] stepper motor driver. Carrier boards for this chip are [available from Pololu].

This crate is a specialized facade of the [Step/Dir] library. Please consider using Step/Dir directly, as it provides drivers for more stepper motor drivers, as well as an interface to abstract over them.

See [Step/Dir] for more documentation and usage examples.

## Status

This driver is currently very basic in its capabilities. Its design is experimental, and more revisions to the API are expected.

## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.

[drv8825]: https://www.ti.com/product/DRV8825
[available from pololu]: https://www.pololu.com/category/154/drv8825-stepper-motor-driver-carriers-high-current
[step/dir]: https://crates.io/crates/step-dir
[zero clause bsd license]: https://opensource.org/licenses/0BSD
[license.md]: https://github.com/braun-embedded/step-dir/blob/master/LICENSE.md
