//! Step/Dir - Universal Stepper Motor Interface
//!
//! Step/Dir aims to provide an interface that abstracts over stepper motor
//! drivers and controllers, exposing high-level hardware features directly
//! where available, or providing software fallbacks where hardware support is
//! lacking.
//!
//! Step/Dir is part of the [Flott] motion control toolkit. Please also check
//! out [RampMaker], a library for generating stepper acceleration ramps. In a
//! future version, both libraries will be integrated, but for now they can be
//! used separately to complement each other.
//!
//! Right now, Step/Dir supports the following drivers:
//!
//! - [DRV8825](crate::drivers::drv8825::DRV8825)
//! - [STSPIN220](crate::drivers::stspin220::STSPIN220)
//!
//! Please check out the documentation of [`Stepper`], which is the main entry
//! point to this API.
//!
//! # Example
//!
//! ``` rust
//! # fn main()
//! #     -> Result<
//! #         (),
//! #         step_dir::Error<
//! #             core::convert::Infallible,
//! #             core::convert::Infallible,
//! #             core::convert::Infallible,
//! #         >
//! #     > {
//! #
//! use step_dir::{
//!     embedded_time::duration::Nanoseconds,
//!     Direction, Stepper,
//! };
//!
//! // This constant defines how much time there is between two steps. Changing
//! // this value directly affects the speed at which the motor runs.
//! const STEP_DELAY: Nanoseconds = Nanoseconds(500_000);
//!
//! # // Use a real driver to make things easy, without making the example seem
//! # // too specific to one driver.
//! # type MyDriver = step_dir::drivers::drv8825::DRV8825<
//! #     (), (), (), (), (), (), (), (), ()
//! # >;
//! #
//! # struct Pin;
//! # impl step_dir::embedded_hal::digital::OutputPin for Pin {
//! #     type Error = core::convert::Infallible;
//! #     fn try_set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn try_set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! #
//! # struct Timer;
//! # impl step_dir::embedded_hal::timer::CountDown for Timer {
//! #     type Error = core::convert::Infallible;
//! #     type Time = Ticks;
//! #     fn try_start<T>(&mut self, count: T) -> Result<(), Self::Error>
//! #         where T: Into<Self::Time>
//! #     {
//! #         Ok(())
//! #     }
//! #     fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
//! #         Ok(())
//! #     }
//! # }
//! #
//! # struct Ticks;
//! # impl From<Nanoseconds> for Ticks {
//! #     fn from(_: Nanoseconds) -> Self {
//! #         Self
//! #     }
//! # }
//! #
//! # fn delay_ns(_: Nanoseconds) {}
//! #
//! // We need some `embedded_hal::digital::OutputPin` implementations connected
//! // to the STEP and DIR signals of our driver chip. How you acquire those
//! // depends on the platform you run on. Here, we'll use a mock implementation
//! // for the sake of demonstration.
//! let step = Pin;
//! let dir = Pin;
//!
//! // We also need a timer (that implements `embedded_hal::timer::CountDown`),
//! // since there are time-critical aspects to communicating with the driver
//! // chip. Again, how you acquire one depends on your target platform, and
//! // again, we'll use a mock here for the sake of demonstration.
//! let mut timer = Timer;
//!
//! // Now we need to initialize the stepper API. We do this by initializing a
//! // driver (`MyDriver`), then wrapping that into the generic API (`Stepper`).
//! // `MyDriver` is a placeholder. In a real use-case, you'd typically use one
//! // of the drivers from the `step_dir::drivers` module, but any driver that
//! // implements the traits from `step_dir::traits` will do.
//! //
//! // By default, drivers can't do anything after being initialized. This means
//! // they also don't require any hardware resources, which makes them easier
//! // to use when you don't need all features.
//! //
//! // Here, we enable control over the STEP and DIR pins, as we want to step
//! // the motor in a defined direction.
//! let mut stepper = Stepper::from_driver(MyDriver::new())
//!     .enable_direction_control(dir, Direction::Forward, &mut timer)?
//!     .enable_step_control(step);
//!
//! // Rotate stepper motor by a few steps.
//! for _ in 0 .. 5 {
//!     // The `step` method returns a future. We just use it to block until the
//!     // operation completes, but you can also use the API in a non-blocking
//!     // way.
//!     stepper.step(&mut timer).wait()?;
//!
//!     // After telling the driver to make a step, we need to make sure to call
//!     // the step method again after an appropriate amount of time. Let's just
//!     // wait for the right time, using this example `delay_ns` function. How
//!     // you do this in your own code is up to you.
//!     delay_ns(STEP_DELAY - stepper.pulse_length());
//! }
//! #
//! # Ok(())
//! # }
//! ```
//!
//! [Flott]: https://flott-motion.org/
//! [RampMaker]: https://crates.io/crates/ramp-maker

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs, broken_intra_doc_links)]

pub extern crate embedded_hal;
pub extern crate embedded_time;

pub mod drivers;
pub mod step_mode;
pub mod traits;

mod stepper;

pub use self::stepper::*;

/// Defines the direction in which to rotate the motor
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// Rotate the motor forward
    ///
    /// This corresponds to whatever direction the motor rotates in when the
    /// driver's DIR signal is set HIGH.
    Forward = 1,

    /// Rotate the motor backward
    ///
    /// This corresponds to whatever direction the motor rotates in when the
    /// driver's DIR signal set is LOW.
    Backward = -1,
}

impl Direction {
    /// Create a `Direction` from a velocity value
    ///
    /// Expects the velocity value to be signed. Zero or a positive value will
    /// result in forward direction, a negative value will result in backward
    /// direction.
    pub fn from_velocity<Velocity>(velocity: Velocity) -> Self
    where
        Velocity: num_traits::Signed,
    {
        if velocity.is_negative() {
            Self::Backward
        } else {
            Self::Forward
        }
    }
}
