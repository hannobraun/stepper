# Step/Dir - Universal Stepper Motor Interface [![crates.io](https://img.shields.io/crates/v/step-dir.svg)](https://crates.io/crates/step-dir) [![Documentation](https://docs.rs/step-dir/badge.svg)](https://docs.rs/step-dir) ![CI Build](https://github.com/flott-motion/step-dir/workflows/CI%20Build/badge.svg)

## About

Step/Dir provides a low-level interface which abstracts over stepper motor drivers that are controlled through STEP and DIR signals. Higher-level code written against its API can control any stepper motor driver supported by Step/Dir.

Step/Dir does not provide any higher-level features like acceleration ramps. It is intended to be a building block for code that implements these higher-level features.

Right now, Step/Dir supports the following drivers:

- [DRV8825] ([crate](https://crates.io/crates/drv8825))
- [STSPIN220] ([crate](https://crates.io/crates/stspin220))

Please check out [the documentation](https://docs.rs/step-dir) to learn more.


## Status

Step/Dir is still under active development. Its API is going to change, as more features and support for more drivers are added.

The library is definitely usable, but hasn't been proven in many use cases yet. If you find any problems, please feel free to open an issue on the GitHub repository.

Step/Dir is maintained by:

- Hanno Braun ([@hannobraun](https://github.com/hannobraun))
- Jesse Braham ([@jessebraham](https://github.com/jessebraham))


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


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.


[DRV8825]: https://www.ti.com/product/DRV8825
[STSPIN220]: https://www.st.com/en/motor-drivers/stspin220.html
[API Reference]: https://docs.rs/step-dir
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: https://github.com/flott-motion/step-dir/blob/master/LICENSE.md
