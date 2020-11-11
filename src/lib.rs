//! Step/Dir - Library for controlling stepper motors
//!
//! Step/Dir provides an abstract interface over drivers libraries for stepper
//! motor drivers. It also contains a reference implementation for the STSPIN220
//! stepper motor driver.
//!
//! It is intended to be a low-level interface to typical stepper motor drivers
//! that are controlled using STEP and DIR pins. It does not contain any higher-
//! level features like acceleration ramps. Rather, it is designed as a low-
//! level building block to be used by higher-level control code.

#![no_std]
#![deny(missing_docs)]

pub extern crate embedded_hal;
pub extern crate embedded_time;

/// Re-exports the traits from this library
pub mod prelude {
    pub use super::{SetStepMode as _, Step as _};
}

#[cfg(feature = "drv8825")]
pub mod drv8825;

#[cfg(feature = "stspin220")]
pub mod stspin220;

mod step_mode;

pub use self::step_mode::*;

use core::convert::TryFrom;

use embedded_time::Clock;

/// Blocking interface for setting the step mode
pub trait SetStepMode {
    /// The error that can occur while using this trait
    type Error;

    /// The type that defines the microstepping mode
    ///
    /// This crate includes a number of enums that can be used for this purpose.
    type StepMode: Into<u16> + TryFrom<u16, Error = InvalidStepModeError>;

    /// Sets the step mode
    fn set_step_mode<Clk: Clock>(
        &mut self,
        step_mode: Self::StepMode,
        clock: &Clk,
    ) -> Result<(), Self::Error>;
}

/// Blocking interface for making single steps
pub trait Step {
    /// The error that can occur while using this trait
    type Error;

    /// Rotates the motor one (micro-)step in the given direction
    ///
    /// This should result in the motor making one step. To achieve a specific
    /// speed, the user must call this method at the appropriate frequency.
    ///
    /// Requires a reference to an `embedded_time::Clock` implementation to
    /// handle the timing. Please make sure that the timer doesn't overflow
    /// while this method is running.
    fn step<Clk: Clock>(
        &mut self,
        dir: Dir,
        clock: &Clk,
    ) -> Result<(), Self::Error>;
}

/// Defines the direction in which to rotate the motor
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dir {
    /// Rotate the motor forward
    ///
    /// This corresponds to whatever direction the motor rotates in when the
    /// dir signal is set HIGH.
    Forward,

    /// Rotate the motor backward
    ///
    /// This corresponds to whatever direction the motor rotates in when the
    /// dir signal set is LOW.
    Backward,
}
