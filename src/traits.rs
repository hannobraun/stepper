//! Contains traits that can be implemented by Step/Dir drivers

use embedded_time::Clock;

use crate::{Dir, StepMode};

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
