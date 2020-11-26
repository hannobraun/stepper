# A4988 Driver [![crates.io](https://img.shields.io/crates/v/a4988.svg)](https://crates.io/crates/a4988) [![Documentation](https://docs.rs/a4988/badge.svg)](https://docs.rs/a4988) ![CI Build](https://github.com/braun-embedded/step-dir/workflows/CI%20Build/badge.svg)

## About

Rust driver crate for the [A4988] stepper motor driver. Carrier boards for this chip are [available from Pololu].

This crate is a specialized facade of the [Step/Dir] library. Please consider using Step/Dir directly, as it provides drivers for more stepper motor drivers, as well as an interface to abstract over them.

See [Step/Dir] for more documentation and usage examples.

## Status

This driver is currently very basic in its capabilities. Its design is experimental, and more revisions to the API are expected.

## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.

[a4988]: https://www.allegromicro.com/en/Products/Motor-Drivers/Brush-DC-Motor-Drivers/A4988
[available from pololu]: https://www.pololu.com/category/156/a4988-stepper-motor-driver-carriers
[step/dir]: (https://crates.io/crates/step-dir)
[zero clause bsd license]: https://opensource.org/licenses/0BSD
[license.md]: https://github.com/braun-embedded/step-dir/blob/master/LICENSE.md
