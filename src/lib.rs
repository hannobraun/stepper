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
    pub use crate::traits::{SetStepMode as _, Step as _};
}

pub mod drivers;
pub mod traits;

mod driver;
mod step_mode;

pub use self::{driver::*, step_mode::*};

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
