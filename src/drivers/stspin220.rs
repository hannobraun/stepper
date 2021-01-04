//! STSPIN220 Driver
//!
//! Platform-agnostic driver for the STSPIN220 stepper motor driver. This module
//! can be used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! The entry point to this module is the [`STSPIN220`] struct.
//!
//! # Example
//!
//! ``` rust
//! # fn main()
//! #     -> Result<
//! #         (),
//! #         step_dir::StepError<core::convert::Infallible>
//! #     > {
//! #
//! use step_dir::{
//!     embedded_time::{duration::Microseconds, Clock as _},
//!     drivers::stspin220::STSPIN220,
//!     Direction, Driver,
//! };
//!
//! const STEP_DELAY: Microseconds = Microseconds(500);
//!
//! # struct Pin;
//! # impl step_dir::embedded_hal::digital::OutputPin for Pin {
//! #     type Error = core::convert::Infallible;
//! #     fn try_set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn try_set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! #
//! # struct Clock(std::time::Instant);
//! # impl step_dir::embedded_time::Clock for Clock {
//! #     type T = u32;
//! #     const SCALING_FACTOR: step_dir::embedded_time::fraction::Fraction =
//! #         step_dir::embedded_time::fraction::Fraction::new(1, 1_000_000);
//! #     fn try_now(&self)
//! #         -> Result<
//! #             step_dir::embedded_time::Instant<Self>,
//! #             step_dir::embedded_time::clock::Error
//! #         >
//! #     {
//! #         Ok(
//! #             step_dir::embedded_time::Instant::new(
//! #                 self.0.elapsed().as_micros() as u32
//! #             )
//! #         )
//! #     }
//! # }
//! #
//! # let step_mode3 = Pin;
//! # let dir_mode4 = Pin;
//! # let mut clock = Clock(std::time::Instant::now());
//! #
//! // You need to acquire the GPIO pins connected to the STEP/MODE3 and
//! // DIR/MODE4 signals. How you do this depends on your target platform. All
//! // the driver cares about is that they implement
//! // `embedded_hal::digital::OutputPin`. You also need an implementation of
//! // `embedded_hal::blocking::DelayUs`.
//!
//! // Create driver API from STEP/MODE3 and DIR/MODE4 pins.
//! let mut driver = Driver::from_inner(STSPIN220::new())
//!     .enable_direction_control(dir_mode4, Direction::Forward, &clock)?
//!     .enable_step_control(step_mode3);
//!
//! // Rotate stepper motor by a few steps.
//! for _ in 0 .. 5 {
//!     let timer = clock.new_timer(STEP_DELAY).start()?;
//!     driver.step(&clock)?;
//!     timer.wait()?;
//! }
//!
//! #
//! # Ok(())
//! # }
//! ```
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal

use embedded_hal::digital::{OutputPin, PinState};
use embedded_time::duration::Nanoseconds;

use crate::{
    traits::{
        EnableDirectionControl, EnableStepControl, EnableStepModeControl,
        SetDirection, SetStepMode, Step,
    },
    StepMode256,
};

/// The STSPIN220 driver API
///
/// You can create an instance of this struct by calling [`STSPIN220::new`]. See
/// [module documentation] for a full example that uses this API.
///
/// [module documentation]: index.html
pub struct STSPIN220<
    EnableFault,
    StandbyReset,
    Mode1,
    Mode2,
    StepMode3,
    DirMode4,
> {
    enable_fault: EnableFault,
    standby_reset: StandbyReset,
    mode1: Mode1,
    mode2: Mode2,
    step_mode3: StepMode3,
    dir_mode4: DirMode4,
}

impl STSPIN220<(), (), (), (), (), ()> {
    /// Create a new instance of `STSPIN220`
    ///
    /// The resulting instance won't be able to do anything yet. You can call
    /// the various `enable_` methods of [`Driver`](crate::Driver) to rectify
    /// that.
    pub fn new() -> Self {
        Self {
            enable_fault: (),
            standby_reset: (),
            mode1: (),
            mode2: (),
            step_mode3: (),
            dir_mode4: (),
        }
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > EnableStepModeControl<(StandbyReset, Mode1, Mode2)>
    for STSPIN220<EnableFault, (), (), (), StepMode3, DirMode4>
where
    StandbyReset: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
    StepMode3: OutputPin<Error = OutputPinError>,
    DirMode4: OutputPin<Error = OutputPinError>,
{
    type WithStepModeControl =
        STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>;

    fn enable_step_mode_control(
        self,
        (standby_reset, mode1, mode2): (StandbyReset, Mode1, Mode2),
    ) -> Self::WithStepModeControl {
        STSPIN220 {
            enable_fault: self.enable_fault,
            standby_reset,
            mode1,
            mode2,
            step_mode3: self.step_mode3,
            dir_mode4: self.dir_mode4,
        }
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > SetStepMode
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
where
    StandbyReset: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
    StepMode3: OutputPin<Error = OutputPinError>,
    DirMode4: OutputPin<Error = OutputPinError>,
{
    const SETUP_TIME: Nanoseconds = Nanoseconds(1_000);
    const HOLD_TIME: Nanoseconds = Nanoseconds(100_000);

    type Error = OutputPinError;
    type StepMode = StepMode256;

    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error> {
        // Force driver into standby mode.
        self.standby_reset.try_set_low()?;

        use PinState::*;
        use StepMode256::*;
        let (mode1, mode2, mode3, mode4) = match step_mode {
            Full => (Low, Low, Low, Low),
            M2 => (High, Low, High, Low),
            M4 => (Low, High, Low, High),
            M8 => (High, High, High, Low),
            M16 => (High, High, High, High),
            M32 => (Low, High, Low, Low),
            M64 => (High, High, Low, High),
            M128 => (High, Low, Low, Low),
            M256 => (High, High, Low, Low),
        };

        // Set mode signals.
        self.mode1.try_set_state(mode1)?;
        self.mode2.try_set_state(mode2)?;
        self.step_mode3.try_set_state(mode3)?;
        self.dir_mode4.try_set_state(mode4)?;

        Ok(())
    }

    fn enable_driver(&mut self) -> Result<(), Self::Error> {
        // Leave standby mode.
        self.standby_reset.try_set_high()
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > EnableDirectionControl<DirMode4>
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, ()>
where
    DirMode4: OutputPin<Error = OutputPinError>,
{
    type WithDirectionControl =
        STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>;

    fn enable_direction_control(
        self,
        dir_mode4: DirMode4,
    ) -> Self::WithDirectionControl {
        STSPIN220 {
            enable_fault: self.enable_fault,
            standby_reset: self.standby_reset,
            mode1: self.mode1,
            mode2: self.mode2,
            step_mode3: self.step_mode3,
            dir_mode4,
        }
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > SetDirection
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
where
    DirMode4: OutputPin<Error = OutputPinError>,
{
    const SETUP_TIME: Nanoseconds = Nanoseconds(100);

    type Dir = DirMode4;
    type Error = OutputPinError;

    fn dir(&mut self) -> &mut Self::Dir {
        &mut self.dir_mode4
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > EnableStepControl<StepMode3>
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, (), DirMode4>
where
    StepMode3: OutputPin<Error = OutputPinError>,
{
    type WithStepControl =
        STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>;

    fn enable_step_control(
        self,
        step_mode3: StepMode3,
    ) -> Self::WithStepControl {
        STSPIN220 {
            enable_fault: self.enable_fault,
            standby_reset: self.standby_reset,
            mode1: self.mode1,
            mode2: self.mode2,
            step_mode3,
            dir_mode4: self.dir_mode4,
        }
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > Step
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
where
    StepMode3: OutputPin<Error = OutputPinError>,
{
    const PULSE_LENGTH: Nanoseconds = Nanoseconds(100);

    type Step = StepMode3;
    type Error = OutputPinError;

    fn step(&mut self) -> &mut Self::Step {
        &mut self.step_mode3
    }
}
