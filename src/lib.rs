//! Step/Dir - Universal Stepper Motor Interface
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
//! Please check out the documentation of [`Driver`], which is the main entry
//! point to this API.

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs, broken_intra_doc_links)]

pub extern crate embedded_hal;
pub extern crate embedded_time;

pub mod drivers;
pub mod step_mode;
pub mod traits;

mod driver;

pub use self::driver::*;

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
