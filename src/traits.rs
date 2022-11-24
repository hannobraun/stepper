//! Traits that can be implemented by Stepper drivers
//!
//! Users are generally not expected to use these traits directly, except to
//! specify trait bounds, where necessary. Please check out [`Stepper`], which
//! uses these traits to provide a unified API.
//!
//! There are two kinds of traits in this module:
//! 1. Those that provide a minimal and low-level interface over a specific
//!    capability (like controlling the microstepping mode).
//! 2. Those that provide an API for enabling these capabilities, taking
//!    ownership of the resources that are required to do so.
//!
//! When constructed, drivers usually do not provide access to any of their
//! capabilities. This means users can specifically enable the capabilities they
//! need, and do not have have to provide hardware resources (like output pins)
//! for capabilities that they are not going to use.
//!
//! This approach also provides a lot of flexibility for non-standard use cases,
//! for example if not all driver capabilities are controlled by software.
//!
//! [`Stepper`]: crate::Stepper

use embedded_hal::digital::OutputPin;
use fugit::NanosDurationU32 as Nanoseconds;

use crate::step_mode::StepMode;

/// Enable microstepping mode control for a driver
///
/// The `Resources` type parameter defines the hardware resources required for
/// controlling microstepping mode.
pub trait EnableStepModeControl<Resources> {
    /// The type of the driver after microstepping mode control has been enabled
    type WithStepModeControl: SetStepMode;

    /// Enable microstepping mode control
    fn enable_step_mode_control(
        self,
        res: Resources,
    ) -> Self::WithStepModeControl;
}

/// Implemented by drivers that support controlling the microstepping mode
pub trait SetStepMode {
    /// The time the mode signals need to be held before re-enabling the driver
    const SETUP_TIME: Nanoseconds;

    /// The time the mode signals need to be held after re-enabling the driver
    const HOLD_TIME: Nanoseconds;

    /// The error that can occur while using this trait
    type Error;

    /// The type that defines the microstepping mode
    ///
    /// This crate includes a number of enums that can be used for this purpose.
    type StepMode: StepMode;

    /// Apply the new step mode configuration
    ///
    /// Typically this puts the driver into reset and sets the mode pins
    /// according to the new step mode.
    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error>;

    /// Re-enable the driver after the mode has been set
    fn enable_driver(&mut self) -> Result<(), Self::Error>;
}

/// Enable direction control for a driver
///
/// The `Resources` type parameter defines the hardware resources required for
/// direction control.
pub trait EnableDirectionControl<Resources> {
    /// The type of the driver after direction control has been enabled
    type WithDirectionControl: SetDirection;

    /// Enable direction control
    fn enable_direction_control(
        self,
        res: Resources,
    ) -> Self::WithDirectionControl;
}

/// Implemented by drivers that support controlling the DIR signal
pub trait SetDirection {
    /// The time that the DIR signal must be held for a change to apply
    const SETUP_TIME: Nanoseconds;

    /// The type of the DIR pin
    type Dir: OutputPin;

    /// The error that can occur while accessing the DIR pin
    type Error;

    /// Provides access to the DIR pin
    fn dir(&mut self) -> Result<&mut Self::Dir, Self::Error>;
}

/// Enable step control for a driver
///
/// The `Resources` type parameter defines the hardware resources required for
/// step control.
pub trait EnableStepControl<Resources> {
    /// The type of the driver after step control has been enabled
    type WithStepControl: Step;

    /// Enable step control
    fn enable_step_control(self, res: Resources) -> Self::WithStepControl;
}

/// Implemented by drivers that support controlling the STEP signal
pub trait Step {
    /// The minimum length of a STEP pulse
    const PULSE_LENGTH: Nanoseconds;

    /// The type of the STEP pin
    type Step: OutputPin;

    /// The error that can occur while accessing the STEP pin
    type Error;

    /// Provides access to the STEP pin
    fn step(&mut self) -> Result<&mut Self::Step, Self::Error>;
}

/// Enable motion control for a driver
///
/// The `Resources` type parameter defines the hardware resources required for
/// motion control.
pub trait EnableMotionControl<Resources, const TIMER_HZ: u32> {
    /// The type of the driver after motion control has been enabled
    type WithMotionControl: MotionControl;

    /// Enable step control
    fn enable_motion_control(self, res: Resources) -> Self::WithMotionControl;
}

/// Implemented by drivers that have motion control capabilities
///
/// A software-based fallback implementation exists in the [`motion_control`]
/// module, for drivers that implement [SetDirection] and [Step].
///
/// [`motion_control`]: crate::motion_control
pub trait MotionControl {
    /// The type used by the driver to represent velocity
    type Velocity: Copy;

    /// The type error that can happen when using this trait
    type Error;

    /// Move to the given position
    ///
    /// This method must arrange for the motion to start, but must not block
    /// until it is completed. If more attention is required during the motion,
    /// this should be handled in [`MotionControl::update`].
    fn move_to_position(
        &mut self,
        max_velocity: Self::Velocity,
        target_step: i32,
    ) -> Result<(), Self::Error>;

    /// Reset internal position to the given value
    ///
    /// This method must not start a motion. Its only purpose is to change the
    /// driver's internal position value, for example for homing.
    fn reset_position(&mut self, step: i32) -> Result<(), Self::Error>;

    /// Update an ongoing motion
    ///
    /// This method may contain any code required to maintain an ongoing motion,
    /// if required, or it might just check whether a motion is still ongoing.
    ///
    /// Return `true`, if motion is ongoing, `false` otherwise. If `false` is
    /// returned, the caller may assume that this method doesn't need to be
    /// called again, until starting another motion.
    fn update(&mut self) -> Result<bool, Self::Error>;
}
