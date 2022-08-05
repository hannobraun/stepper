# Platform Support Guide

## Introduction

Stepper is designed to run pretty much everywhere, including very limited target platforms like microcontrollers. To that end, it doesn't interface with any platform-specific code, instead relying on a set of traits that must be implemented on a given target platform for Stepper to run.

Unfortunately, the set of traits required by Stepper is somewhat non-standard, and not implemented on most target platforms. The remainder of this guide will explain what those traits are, and how they can be implemented.

At the time of writing (April 2021), the only target platform that Stepper is confirmed to work on is the [LPC845][LPC845] by NXP. Porting it to other microcontrollers should be relatively straight-forward, by following the rest of this guide.

If you just want to give Stepper a try and don't have specific hardware in mind, just grab an [LPC845-BRK] development board, which is fully supported. [Stepper Terminal](https://github.com/braun-embedded/stepper-terminal) should provide a good starting point.

If you find any problems with this guide, if anything is unclear, incomplete, or incorrect, please [open an issue](https://github.com/braun-embedded/stepper/issues). If you need any help with adding support for Stepper to a given HAL, please feel free to reach out to [@hannobraun](https://github.com/hannobraun), for example by pinging him from an issue or pull request in the HAL's repository.


## Required Traits

As explained above, Stepper relies on a set of traits to interface with its target platforms. These traits need to be implemented on a given target platform for Stepper to work.

Stepper relies on the following implementations from the [`embedded-hal`](https://crates.io/crates/embedded-hal) crate:

- [`embedded_hal::digital::OutputPin`](https://docs.rs/embedded-hal/1.0.0-alpha.4/embedded_hal/digital/trait.OutputPin.html) - Used for interfacing with driver chips, for example STEP, DIR, and any other digital signals.
- [`embedded_hal::timer::CountDown`](https://docs.rs/embedded-hal/1.0.0-alpha.4/embedded_hal/timer/trait.CountDown.html) - Used for any timing-related tasks.

`embedded-hal` is widely supported in the Embedded Rust ecosystem, but Stepper depends on its latest version (1.0.0-alpha.4, at the time of writing), which is not widely supported.

In addition, Stepper relies on an implementation of `TryFrom<Nanoseconds>` for `CountDown::Time`. Let's unpack this:

- [`TryFrom`](https://doc.rust-lang.org/core/convert/trait.TryFrom.html) is a standard trait from the Rust core library, widely used for fallible conversions.
- [`Nanoseconds`](https://docs.rs/embedded-time/0.10.1/embedded_time/duration/struct.Nanoseconds.html) is a type from [`embedded-time`](https://crates.io/crates/embedded-time), a library for handling time on embedded systems.
- `CountDown::Time` is an associated type of the aforementioned `CountDown` trait. Stepper expects a conversion from `Nanoseconds` to this associated type to exist.


## Adding support to HAL libraries

In Rust, Hardware Abstraction Layer (HAL) refers to a library that provides a high-level interface a specific microcontroller (or family thereof). An example of such a HAL is [LPC8xx HAL]. These HAL libraries are the ideal place for the trait implementations required by Stepper.

For most HAL libraries, two main hurdles need to be overcome, before these implementations can be added:

- Adding support for the latest alpha version of `embedded-hal`.
- Adding support for `embedded-time`.

### `embedded-hal` alpha

Many HALs plan to switch to `embedded-hal` 1.0 once it comes out, and even have open pull requests (written against an alpha version) to that effect. However, it is possible (and perfectly practical) to use a more gradual approach and support the latest stable version of `embedded-hal` and the latest alpha version side by side.

This approach has been implemented in [LPC8xx HAL]. The following list lines out how to repeat this for other HAL libraries:

1. Add a dependency to the latest `embedded-hal` version to `Cargo.toml`. Use the `package` attribute to refer to it by another name, to prevent name collision ([example](https://github.com/lpc-rs/lpc8xx-hal/blob/a2b774e8a9ef025fb5119ddfb09e1b190e510896/Cargo.toml#L44-L46)).
2. Import the traits into the module where they should be implemented. Change their name using `as` to prevent name collisions ([example](https://github.com/lpc-rs/lpc8xx-hal/blob/a2b774e8a9ef025fb5119ddfb09e1b190e510896/src/gpio.rs#L49-L53)).
3. Implement the traits next to their non-alpha versions ([example](https://github.com/lpc-rs/lpc8xx-hal/blob/a2b774e8a9ef025fb5119ddfb09e1b190e510896/src/gpio.rs#L767-L782)).

None of this is hard, but some HAL maintainers might prefer not to add a dependency on an alpha version. The main drawback of this approach is that it requires ongoing updates, as new `embedded-hal` alpha versions come out.

However, besides providing support for Stepper, the big advantage is that the transition to the new `embedded-hal` version can be gradual. It also can result in more testing of the new `embedded-hal` version.

### `embedded-time`

Unfortunately `embedded-time` isn't widely supported yet in the ecosystem. Most HALs do have duration types though that duplicate what `embedded-time` already provides (often in a module called `time`). In many cases, adding support for `embedded-time` might be as simple as adding the dependency, removing the `time` module, and updating any code that no longer compiles.

Please note though that Stepper doesn't require this. All it requires is the single `TryFrom<Nanoseconds>` implementation for the `Time` associated type of the `CountDown` implementation that is provided to Stepper.

Again, this has been implemented in [LPC8xx HAL]. The following list lines out how to do it for other HALs:

1. Add a dependency on the latest version of `embedded-time` ([example](https://github.com/lpc-rs/lpc8xx-hal/blob/a2b774e8a9ef025fb5119ddfb09e1b190e510896/Cargo.toml#L27)).
2. Add a dedicated type to represent time for the implementation of `CountDown` that you want to use ([example](https://github.com/lpc-rs/lpc8xx-hal/blob/a2b774e8a9ef025fb5119ddfb09e1b190e510896/src/mrt/ticks.rs#L30)). Most HALs just use a type like `u32` which is not very descriptive, and quite limiting in what can be done with it.
3. Set the `CountDown` implementation's `Time` associated type to the dedicated time type you created ([example](https://github.com/lpc-rs/lpc8xx-hal/blob/master/src/mrt/channel.rs#L81)).
4. Implement the conversion from `Nanoseconds` to timer ticks ([example](https://github.com/lpc-rs/lpc8xx-hal/blob/master/src/mrt/ticks.rs#L116-L126)). Unfortunately the only example available is quite simplistic (the timer is hardcoded to one specific frequency, and that specific conversion can't fail). It should be possible to support more complex cases using const generics or other techniques.

This is not that easy (and can in fact get quite complicated, if the timer frequency is configurable), but this has lots of utility beyond supporting Stepper.


## Workaround: `compat` module

If adding the required trait implementations directly in the HAL is not practical for some reason, you can work around this by providing these implementations in your own code. While it is not possible to implement a foreign trait for a foreign type ("foreign" as in "defined in another crate"), you can create your own wrapper types, and implement the required traits for them.

The `compat` module in Stepper provides such wrappers.


[LPC845]: https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/low-cost-microcontrollers-mcus-based-on-arm-cortex-m0-plus-cores:LPC84X
[LPC845-BRK]: https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/lpc845-breakout-board-for-lpc84x-family-mcus:LPC845-BRK
[LPC8xx HAL]: https://crates.io/crates/lpc8xx-hal
