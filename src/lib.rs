//! Stepper - Universal Stepper Motor Interface
//!
//! Stepper aims to provide an interface that abstracts over stepper drivers and
//! motion control chips, exposing high-level hardware features directly where
//! available, or providing software fallbacks where hardware support is
//! lacking. Stepper is part of the [Flott] motion control toolkit.
//!
//! Right now, Stepper supports the following ICs:
//!
//! - [DRV8825](crate::drivers::drv8825::DRV8825)
//! - [STSPIN220](crate::drivers::stspin220::STSPIN220)
//! - [DQ542MA](crate::drivers::dq542ma::DQ542MA)
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
//! #         stepper::Error<
//! #             core::convert::Infallible,
//! #             core::convert::Infallible,
//! #             core::convert::Infallible,
//! #             core::convert::Infallible,
//! #         >
//! #     > {
//! #
//! use stepper::{
//!     fugit::NanosDurationU32 as Nanoseconds,
//!     motion_control, ramp_maker,
//!     Direction, Stepper,
//! };
//!
//! # // Use a real driver to make things easy, without making the example seem
//! # // too specific to one driver.
//! # type MyDriver = stepper::drivers::drv8825::DRV8825<
//! #     (), (), (), (), (), (), (), (), ()
//! # >;
//! #
//! # struct Pin;
//! # impl embedded_hal::digital::ErrorType for Pin {
//! #     type Error = core::convert::Infallible;
//! # }
//! # impl stepper::embedded_hal::digital::blocking::OutputPin for Pin {
//! #     fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! #
//! # struct Timer<const TIMER_HZ: u32>;
//! #     impl<const TIMER_HZ: u32> Timer<TIMER_HZ> {
//! #         pub fn new() -> Self {  Self {} }
//! #     }
//! #     impl<const TIMER_HZ: u32> fugit_timer::Timer<TIMER_HZ> for Timer<TIMER_HZ>{
//! #         type Error = std::convert::Infallible;
//! #         fn now(&mut self) -> fugit::TimerInstantU32<TIMER_HZ> {
//! #             todo!()
//! #         }
//! #         fn start(&mut self, _duration: fugit::TimerDurationU32<TIMER_HZ>) -> Result<(), Self::Error> {
//! #             Ok(())
//! #         }
//! #         fn cancel(&mut self) -> Result<(), Self::Error> {
//! #             todo!()
//! #         }
//! #         fn wait(&mut self) -> nb::Result<(), Self::Error> {
//! #             Ok(())
//! #         }
//! #     }
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
//! let mut timer = Timer::<1_000_000>::new();
//!
//! // Define the numeric type we're going to use. We'll use a fixed-point type
//! // here, as that's the most widely supported. If your target hardware has
//! // support for floating point, it might be more convenient (and possibly
//! // efficient) to use that instead.
//! type Num = fixed::FixedI64<typenum::U32>;
//!
//! // Define the target acceleration and maximum speed using timer ticks as the
//! // unit of time. We could also use seconds or any other unit of time
//! // (Stepper doesn't care), but then we'd need to provide a conversion from
//! // seconds to timer ticks. This way, we save that conversion.
//! //
//! // These values assume a 1 MHz timer, but that depends on the timer you're
//! // using, of course.
//! let target_accel = Num::from_num(0.001); // steps / tick^2; 1000 steps / s^2
//! let max_speed = Num::from_num(0.001); // steps / tick; 1000 steps / s
//!
//! // We want to use the high-level motion control API (see below), but let's
//! // assume the driver we use for this example doesn't provide hardware
//! // support for that. Let's instantiate a motion profile from the RampMaker
//! // library to provide a software fallback.
//! let profile = ramp_maker::Trapezoidal::new(target_accel);
//!
//! // Now we need to initialize the stepper API. We do this by initializing a
//! // driver (`MyDriver`), then wrapping that into the generic API (`Stepper`).
//! // `MyDriver` is a placeholder. In a real use-case, you'd typically use one
//! // of the drivers from the `stepper::drivers` module, but any driver that
//! // implements the traits from `stepper::traits` will do.
//! //
//! // By default, drivers can't do anything after being initialized. This means
//! // they also don't require any hardware resources, which makes them easier
//! // to use when you don't need all features.
//! let mut stepper = Stepper::from_driver(MyDriver::new())
//!     // Enable direction control
//!     .enable_direction_control(dir, Direction::Forward, &mut timer)?
//!     // Enable step control
//!     .enable_step_control(step)
//!     // Enable motion control using the software fallback
//!     .enable_motion_control((timer, profile, DelayToTicks));
//!
//! // Tell the motor to move 2000 steps (10 revolutions on a typical stepper
//! // motor), while respecting the maximum speed. Since we selected a
//! // trapezoidal motion profile above, this will result in a controlled
//! // acceleration to the maximum speed, and a controlled deceleration after.
//! let target_step = 2000;
//! stepper
//!     .move_to_position(max_speed, target_step)
//!     .wait()?;
//!
//! // Here's the converter that Stepper is going to use internally, to convert
//! // from the computed delay value to timer ticks. Since we chose to use timer
//! // ticks as the unit of time for velocity and acceleration, this conversion
//! // is pretty simple (and cheap).
//! use num_traits::cast::ToPrimitive;
//! pub struct DelayToTicks;
//! impl<const TIMER_HZ: u32> motion_control::DelayToTicks<Num, TIMER_HZ> for DelayToTicks {
//!     type Error = core::convert::Infallible;
//!
//!     fn delay_to_ticks(&self, delay: Num)
//!         -> Result<fugit::TimerDurationU32<TIMER_HZ>, Self::Error>
//!     {
//!         Ok(fugit::TimerDurationU32::<TIMER_HZ>::from_ticks(Num::to_u32(&delay).expect("the delay to convert")))
//!     }
//! }
//! #
//! # Ok(())
//! # }
//! ```
//!
//! [Flott]: https://flott-motion.org/
//! [RampMaker]: https://crates.io/crates/ramp-maker

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

pub extern crate embedded_hal;
pub extern crate fugit;
pub extern crate ramp_maker;

pub mod compat;
pub mod drivers;
pub mod motion_control;
pub mod step_mode;
pub mod traits;
pub mod util;

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
