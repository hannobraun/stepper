//! Contains traits that can be implemented by Step/Dir drivers

use embedded_hal::digital::OutputPin;
use embedded_time::{duration::Nanoseconds, Clock};

use crate::StepMode;

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
    /// The time that the DIR signal must be held for a change to apply
    const SETUP_TIME: Nanoseconds;

    /// The minimum length of a STEP pulse
    const PULSE_LENGTH: Nanoseconds;

    /// The type of the DIR pin
    type Dir: OutputPin<Error = Self::Error>;

    /// The type of the STEP pin
    type Step: OutputPin<Error = Self::Error>;

    /// The error that can occur while using this trait
    type Error;

    /// Provides access to the DIR pin
    fn dir(&mut self) -> &mut Self::Dir;

    /// Provides access to the STEP pin
    fn step(&mut self) -> &mut Self::Step;
}
