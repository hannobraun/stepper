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

#[cfg(feature = "stspin220")]
pub mod stspin220;

use core::convert::TryFrom;

use embedded_hal::digital::PinState;
use embedded_time::Clock;

/// Blocking interface for setting the step mode
pub trait SetStepMode {
    /// The error that can occur while using this trait
    type Error;

    /// Sets the step mode
    fn set_step_mode<Clk: Clock>(
        &mut self,
        step_mode: StepMode,
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

/// Defines the step mode
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StepMode {
    /// Full steps
    Full = 1,

    /// 2 microsteps per full step
    M2 = 2,

    /// 4 microsteps per full step
    M4 = 4,

    /// 8 microsteps per full step
    M8 = 8,

    /// 16 microsteps per full step
    M16 = 16,

    /// 32 microsteps per full step
    M32 = 32,

    /// 64 microsteps per full step
    M64 = 64,

    /// 128 microsteps per full step
    M128 = 128,

    /// 256 microsteps per full step
    M256 = 256,
}

impl StepMode {
    /// Provides the pin signals for the given step mode
    pub fn to_signals(&self) -> (PinState, PinState, PinState, PinState) {
        use PinState::*;
        use StepMode::*;
        match self {
            Full => (Low, Low, Low, Low),
            M2 => (High, Low, High, Low),
            M4 => (Low, High, Low, High),
            M8 => (High, High, High, Low),
            M16 => (High, High, High, High),
            M32 => (Low, High, Low, Low),
            M64 => (High, High, Low, High),
            M128 => (High, Low, Low, Low),
            M256 => (High, High, Low, Low),
        }
    }
}

impl TryFrom<u16> for StepMode {
    type Error = InvalidStepModeError;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(StepMode::Full),
            2 => Ok(StepMode::M2),
            4 => Ok(StepMode::M4),
            8 => Ok(StepMode::M8),
            16 => Ok(StepMode::M16),
            32 => Ok(StepMode::M32),
            64 => Ok(StepMode::M64),
            128 => Ok(StepMode::M128),
            256 => Ok(StepMode::M256),

            _ => Err(InvalidStepModeError),
        }
    }
}

/// Indicates that a given step mode value did not represent a valid step mode
///
/// Valid values are 1, 2, 4, 8, 16, 32, 64, 128, and 256.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidStepModeError;
