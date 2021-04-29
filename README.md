# Stepper - Universal Stepper Motor Interface

[![crates.io](https://img.shields.io/crates/v/stepper.svg)](https://crates.io/crates/stepper) [![Documentation](https://docs.rs/stepper/badge.svg)](https://docs.rs/stepper) ![CI Build](https://github.com/flott-motion/stepper/workflows/CI%20Build/badge.svg)

**Please consider supporting this project financially. More information below.**

## About

Stepper aims to provide an interface that abstracts over stepper motor drivers and controllers, exposing high-level hardware features directly where available, or providing software fallbacks where hardware support is lacking.

Stepper is part of the [Flott] motion control toolkit. Please also check out [RampMaker], a library for generating stepper acceleration ramps. In a future version, both libraries will be integrated, but for now they can be used separately to complement each other.

Right now, Stepper supports the following drivers:

- [DRV8825] ([crate](https://crates.io/crates/drv8825))
- [STSPIN220] ([crate](https://crates.io/crates/stspin220))

Support for more stepper drivers and controllers will be added in the future. Please consider helping out with this effort, if you need support for a driver or controller that is currently missing.

Please refer to the [API Reference](https://docs.rs/stepper) or one of the following guides to learn more:

- [How to Write a Driver](https://github.com/flott-motion/stepper/tree/main/documentation/how-to-write-a-driver.md)
- [Platform Support Guide](https://github.com/flott-motion/stepper/tree/main/documentation/platform-support.md)


## Status

Stepper is under active development. Its API is going to change, as more features are added and existing ones are improved. Support for drivers is very limited right now, and support for controllers is non-existent.

The library is usable, but far from mature. There are some known limitations that are documented on the [issue tracker](https://github.com/flott-motion/stepper/issues). If you find any additional problems, please open an issue on the GitHub repository.

Stepper is maintained by:

- Hanno Braun ([@hannobraun])
- Jesse Braham ([@jessebraham])


## Usage

Stepper is a library written in Rust and designed for use in Rust projects. It will run on any platform supported by Rust, including microcontrollers.

Add Stepper to your `Cargo.toml` like this:

``` toml
[dependencies.stepper]
version = "0.5" # make sure this is the latest version
```

If you just need to use a specific stepper driver, you can also depend on the crate for that specific driver. For example:

``` toml
[dependencies.drv8825]
version = "0.5" # make sure this is the latest version
```

Please refer to the [API Reference] for more information.


## Funding

If you're getting value out of Stepper or other libraries from the [Flott] toolkit, please consider supporting us financially. Your sponsorship helps to keep the project healthy and moving forward.

[Hanno Braun][@hannobraun], maintainer and original creator of this library, is [accepting sponsorship](https://github.com/sponsors/hannobraun).


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.


[Flott]: https://flott-motion.org/
[RampMaker]: https://crates.io/crates/ramp-maker
[DRV8825]: https://www.ti.com/product/DRV8825
[STSPIN220]: https://www.st.com/en/motor-drivers/stspin220.html
[API Reference]: https://docs.rs/stepper
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: https://github.com/flott-motion/stepper/blob/main/LICENSE.md

[@hannobraun]: https://github.com/hannobraun
[@jessebraham]: https://github.com/jessebraham
