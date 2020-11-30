# Step/Dir - Interface to stepper motor drivers [![crates.io](https://img.shields.io/crates/v/step-dir.svg)](https://crates.io/crates/step-dir) [![Documentation](https://docs.rs/step-dir/badge.svg)](https://docs.rs/step-dir) ![CI Build](https://github.com/braun-embedded/step-dir/workflows/CI%20Build/badge.svg)

## About

Step/Dir provides a low-level interface which abstracts over stepper motor drivers that are controlled through STEP and DIR signals. Higher-level code written against its API can control any stepper motor driver supported by Step/Dir.

Step/Dir does not provide any higher-level features like acceleration ramps. It is intended to be a building block for code that implements these higher-level features.

Right now, Step/Dir supports the following drivers:

- [DRV8825]
- [STSPIN220]

Please check out [the documentation](https://docs.rs/step-dir) to learn more.


## Status

Step/Dir is usable, but still under active development. Its API is going to change, as more features and support for more drivers are added.


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.


[DRV8825]: https://www.ti.com/product/DRV8825
[STSPIN220]: https://www.st.com/en/motor-drivers/stspin220.html
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: https://github.com/braun-embedded/step-dir/blob/master/LICENSE.md
