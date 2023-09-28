//! TMC2099 Driver
//!
//! This can have a UART controller to control a lot of the functionality
//! through software, but it still needs to be implemented
//!
//! Platform-agnostic driver API for the TMC2099 stepper motor driver. Can be
//! used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! For the most part, users are not expected to use this API directly. Please
//! check out [`Stepper`](crate::Stepper) instead.
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal

use core::convert::Infallible;

use embedded_hal::digital::{OutputPin, PinState};
use fugit::NanosDurationU32 as Nanoseconds;

use crate::{
    step_mode::StepMode64,
    traits::{
        EnableDirectionControl, EnableStepControl, EnableStepModeControl,
        SetDirection, SetStepMode, Step as StepTrait,
    },
};

/// The TMC2099 driver API
///
/// Users are not expected to use this API directly, except to create an
/// instance using [`TMC2099::new`]. Please check out
/// [`Stepper`](crate::Stepper) instead.
// Pins left out- SPREAD, CLK- tie to gnd for internal supply, PDN_UART, VREF
// For UART Config use address 0..3 instead of MS pins
pub struct TMC2099<ENN, STDBY, DIAG, MS1_AD0, MS2_AD1, STEP, DIR> {
    enable_n: ENN,
    standby: STDBY,
    diagnostic: DIAG,
    mode0: MS1_AD0,
    mode1: MS2_AD1,
    step: STEP,
    dir: DIR,
}

impl TMC2099<(), (), (), (), (), (), ()> {
    /// Create a new instance of `TMC2099`
    pub fn new() -> Self {
        Self {
            enable_n: (),
            standby: (),
            diagnostic: (),
            mode0: (),
            mode1: (),
            step: (),
            dir: (),
        }
    }
}

impl<ENN, STDBY, MS1_AD0, MS2_AD1, Step, Dir, OutputPinError>
    EnableStepModeControl<(ENN, STDBY, MS1_AD0, MS2_AD1)>
    for TMC2099<(), (), (), (), (), Step, Dir>
where
    ENN: OutputPin<Error = OutputPinError>,
    STDBY: OutputPin<Error = OutputPinError>,
    MS1_AD0: OutputPin<Error = OutputPinError>,
    MS2_AD1: OutputPin<Error = OutputPinError>,
{
    type WithStepModeControl =
        TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, Dir>;

    fn enable_step_mode_control(
        self,
        (enable_n, standby, mode0, mode1): (ENN, STDBY, MS1_AD0, MS2_AD1),
    ) -> Self::WithStepModeControl {
        TMC2099 {
            enable_n,
            standby,
            diagnostic: self.diagnostic,
            mode0,
            mode1,
            step: self.step,
            dir: self.dir,
        }
    }
}

impl<ENN, STDBY, MS1_AD0, MS2_AD1, Step, Dir, OutputPinError> SetStepMode
    for TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, Dir>
where
    ENN: OutputPin<Error = OutputPinError>,
    STDBY: OutputPin<Error = OutputPinError>,
    MS1_AD0: OutputPin<Error = OutputPinError>,
    MS2_AD1: OutputPin<Error = OutputPinError>,
{
    // Timing Requirements (page 6)
    // https://www.pololu.com/file/0J450/A4988.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds::from_ticks(20);
    const HOLD_TIME: Nanoseconds = Nanoseconds::from_ticks(20);

    type Error = OutputPinError;
    type StepMode = StepMode64;

    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error> {
        // Reset the device's internal logic and disable the h-bridge drivers.
        self.enable_n.set_high()?;
        self.standby.set_high()?;

        use PinState::*;
        use StepMode64::*;
        let (mode0, mode1) = match step_mode {
            M8 => (Low, Low),
            M32 => (Low, High),
            M64 => (High, Low),
            M16 => (High, High),
            _ => (High, Low),
        };

        // Set mode signals.
        self.mode0.set_state(mode0)?;
        self.mode1.set_state(mode1)?;

        Ok(())
    }

    fn enable_driver(&mut self) -> Result<(), Self::Error> {
        self.enable_n.set_low()?;
        self.standby.set_low()
    }
}

impl<ENN, STDBY, MS1_AD0, MS2_AD1, Step, Dir, OutputPinError>
    EnableDirectionControl<Dir>
    for TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, ()>
where
    Dir: OutputPin<Error = OutputPinError>,
{
    type WithDirectionControl =
        TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, Dir>;

    fn enable_direction_control(self, dir: Dir) -> Self::WithDirectionControl {
        TMC2099 {
            enable_n: self.enable_n,
            standby: self.standby,
            diagnostic: self.diagnostic,
            mode0: self.mode0,
            mode1: self.mode1,
            step: self.step,
            dir,
        }
    }
}

impl<ENN, STDBY, MS1_AD0, MS2_AD1, Step, Dir, OutputPinError> SetDirection
    for TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, Dir>
where
    Dir: OutputPin<Error = OutputPinError>,
{
    // Timing Requirements (page 6)
    // https://www.pololu.com/file/0J450/A4988.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds::from_ticks(20);

    type Dir = Dir;
    type Error = Infallible;

    fn dir(&mut self) -> Result<&mut Self::Dir, Self::Error> {
        Ok(&mut self.dir)
    }
}

impl<ENN, STDBY, MS1_AD0, MS2_AD1, Step, Dir, OutputPinError>
    EnableStepControl<Step>
    for TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, Dir>
where
    Step: OutputPin<Error = OutputPinError>,
{
    type WithStepControl = TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, Dir>;

    fn enable_step_control(self, step: Step) -> Self::WithStepControl {
        TMC2099 {
            enable_n: self.enable_n,
            standby: self.standby,
            diagnostic: self.diagnostic,
            mode0: self.mode0,
            mode1: self.mode1,
            step,
            dir: self.dir,
        }
    }
}

impl<ENN, STDBY, MS1_AD0, MS2_AD1, Step, Dir, OutputPinError> StepTrait
    for TMC2099<ENN, STDBY, (), MS1_AD0, MS2_AD1, Step, Dir>
where
    Step: OutputPin<Error = OutputPinError>,
{
    // Timing Requirements (page 63)
    // TODO: Add URL
    // min of max(filtering_time, fclk+20)- went with typical val
    const PULSE_LENGTH: Nanoseconds = Nanoseconds::from_ticks(100);

    type Step = Step;
    type Error = Infallible;

    fn step(&mut self) -> Result<&mut Self::Step, Self::Error> {
        Ok(&mut self.step)
    }
}
