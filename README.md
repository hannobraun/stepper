# Step/Dir - Universal Stepper Motor Interface

[![crates.io](https://img.shields.io/crates/v/step-dir.svg)](https://crates.io/crates/step-dir) [![Documentation](https://docs.rs/step-dir/badge.svg)](https://docs.rs/step-dir) ![CI Build](https://github.com/flott-motion/step-dir/workflows/CI%20Build/badge.svg)

**Please consider supporting this project financially. More information below.**

## About

Step/Dir aims to provide an interface that abstracts over stepper motor drivers and controllers, exposing high-level hardware features directly where available, or providing software fallbacks where hardware support is lacking.

Right now, Step/Dir supports the following drivers:

- [DRV8825] ([crate](https://crates.io/crates/drv8825))
- [STSPIN220] ([crate](https://crates.io/crates/stspin220))

Support for more stepper drivers and controllers will be added in the future. Please consider helping out with this effort, if you need support for a driver or controller that is currently missing.

Please check out [the documentation](https://docs.rs/step-dir) to learn more.


## Status

Step/Dir is under active development. Its API is going to change, as more features are added and existing ones are improved. Support for drivers is very limited right now, and support for controllers is non-existent.

The library is usable, but far from mature. If you find any problems, please open an issue on the GitHub repository.

Step/Dir is maintained by:

- Hanno Braun ([@hannobraun])
- Jesse Braham ([@jessebraham])


## Usage

Step/Dir is a library written in Rust and designed for use in Rust projects. It will run on any platform supported by Rust, including microcontrollers.

Add Step/Dir to your `Cargo.toml` like this:

``` toml
[dependencies.step-dir]
version = "0.4" # make sure this is the latest version
```

If you just need to use a specific stepper driver, you can also depend on the crate for that specific driver. For example:

``` toml
[dependencies.drv8825]
version = "0.4" # make sure this is the latest version
```

Please refer to the [API Reference] for more information.


## Funding

If you're getting value out of Step/Dir, please consider supporting us financially. Your sponsorship helps to keep the project healthy and moving forward.

[Hanno Braun][@hannobraun], maintainer and original creator of this library, is [accepting sponsorship](https://github.com/sponsors/hannobraun).


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.


[DRV8825]: https://www.ti.com/product/DRV8825
[STSPIN220]: https://www.st.com/en/motor-drivers/stspin220.html
[API Reference]: https://docs.rs/step-dir
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: https://github.com/flott-motion/step-dir/blob/main/LICENSE.md

[@hannobraun]: https://github.com/hannobraun
[@jessebraham]: https://github.com/jessebraham
