//! Step/Dir - Library for controlling stepper motors
//!
//! Step/Dir provides a low-level interface which abstracts over stepper motor
//! drivers that are controlled through STEP and DIR signals. Higher-level code
//! written against its API can control any stepper motor driver supported by
//! Step/Dir.
//!
//! Step/Dir does not provide any higher-level features like acceleration ramps.
//! It is intended to be a building block for code that implements these higher-
//! level features.
//!
//! Right now, Step/Dir supports the following drivers:
//!
//! - [DRV8825](crate::drivers::drv8825::DRV8825)
//! - [STSPIN220](crate::drivers::stspin220::STSPIN220)
//!
//! Step/Dir defines traits that allow users to write code that is completely
//! agnostic to the stepper motor driver it controls. Currently these traits are
//! limited to *use* of the stepper motor drivers. There are no traits to
//! abstract over driver *initialization*, which still requires driver-specific
//! code.

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs, broken_intra_doc_links)]

pub extern crate embedded_hal;
pub extern crate embedded_time;

/// Re-exports the traits from this library
pub mod prelude {
    pub use super::{SetStepMode as _, Step as _};
}

pub mod drivers;

mod driver;
mod step_mode;

pub use self::{driver::*, step_mode::*};

use embedded_time::Clock;

/// Blocking interface for setting the step mode
pub trait SetStepMode {
    /// The error that can occur while using this trait
    type Error;

    /// The type that defines the microstepping mode
    ///
    /// This crate includes a number of enums that can be used for this purpose.
    type StepMode: StepMode;

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
    Forward = 1,

    /// Rotate the motor backward
    ///
    /// This corresponds to whatever direction the motor rotates in when the
    /// dir signal set is LOW.
    Backward = -1,
}
