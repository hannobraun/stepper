# STSPIN220 Driver [![crates.io](https://img.shields.io/crates/v/stspin220.svg)](https://crates.io/crates/stspin220) [![Documentation](https://docs.rs/stspin220/badge.svg)](https://docs.rs/stspin220) ![CI Build](https://github.com/braun-embedded/step-dir/workflows/CI%20Build/badge.svg)

## About

Rust driver crate for the [STSPIN220] stepper motor driver. Carrier boards for this chip are [available from Pololu].

This crate is a specialized facade for the [Step/Dir] library. Please consider using Step/Dir directly, as it provides drivers for more stepper motor drivers, as well as an interface to abstract over them.

See [Step/Dir] for more documentation and usage examples.

## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.

[stspin220]: https://www.st.com/en/motor-drivers/stspin220.html
[available from pololu]: https://www.pololu.com/category/260/
[step/dir]: https://crates.io/crates/step-dir
[zero clause bsd license]: https://opensource.org/licenses/0BSD
[license.md]: https://github.com/braun-embedded/step-dir/blob/master/LICENSE.md
