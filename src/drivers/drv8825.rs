//! DRV8825 Driver
//!
//! Platform-agnostic driver API for the DRV8825 stepper motor driver. Can be
//! used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! For the most part, users are not expected to use this API directly. Please
//! check out [`Stepper`](crate::Stepper) instead.
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal

use core::convert::Infallible;

use embedded_hal::digital::{blocking::OutputPin, PinState};
use fugit::NanosDurationU32 as Nanoseconds;

use crate::{
    step_mode::StepMode32,
    traits::{
        EnableDirectionControl, EnableStepControl, EnableStepModeControl,
        SetDirection, SetStepMode, Step as StepTrait,
    },
};

/// The DRV8825 driver API
///
/// Users are not expected to use this API directly, except to create an
/// instance using [`DRV8825::new`]. Please check out
/// [`Stepper`](crate::Stepper) instead.
pub struct DRV8825<Enable, Fault, Sleep, Reset, Mode0, Mode1, Mode2, Step, Dir>
{
    enable: Enable,
    fault: Fault,
    sleep: Sleep,
    reset: Reset,
    mode0: Mode0,
    mode1: Mode1,
    mode2: Mode2,
    step: Step,
    dir: Dir,
}

impl DRV8825<(), (), (), (), (), (), (), (), ()> {
    /// Create a new instance of `DRV8825`
    pub fn new() -> Self {
        Self {
            enable: (),
            fault: (),
            sleep: (),
            reset: (),
            mode0: (),
            mode1: (),
            mode2: (),
            step: (),
            dir: (),
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError>
    EnableStepModeControl<(Reset, Mode0, Mode1, Mode2)>
    for DRV8825<(), (), (), (), (), (), (), Step, Dir>
where
    Reset: OutputPin<Error = OutputPinError>,
    Mode0: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
{
    type WithStepModeControl =
        DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>;

    fn enable_step_mode_control(
        self,
        (reset, mode0, mode1, mode2): (Reset, Mode0, Mode1, Mode2),
    ) -> Self::WithStepModeControl {
        DRV8825 {
            enable: self.enable,
            fault: self.fault,
            sleep: self.sleep,
            reset,
            mode0,
            mode1,
            mode2,
            step: self.step,
            dir: self.dir,
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> SetStepMode
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Reset: OutputPin<Error = OutputPinError>,
    Mode0: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
{
    // 7.6 Timing Requirements (page 7)
    // https://www.ti.com/lit/ds/symlink/drv8825.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds::from_ticks(650);
    const HOLD_TIME: Nanoseconds = Nanoseconds::from_ticks(650);

    type Error = OutputPinError;
    type StepMode = StepMode32;

    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error> {
        // Reset the device's internal logic and disable the h-bridge drivers.
        self.reset.set_low()?;

        use PinState::*;
        use StepMode32::*;
        let (mode0, mode1, mode2) = match step_mode {
            Full => (Low, Low, Low),
            M2 => (High, Low, Low),
            M4 => (Low, High, Low),
            M8 => (High, High, Low),
            M16 => (Low, Low, High),
            M32 => (High, High, High),
        };

        // Set mode signals.
        self.mode0.set_state(mode0)?;
        self.mode1.set_state(mode1)?;
        self.mode2.set_state(mode2)?;

        Ok(())
    }

    fn enable_driver(&mut self) -> Result<(), Self::Error> {
        self.reset.set_high()
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError>
    EnableDirectionControl<Dir>
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, ()>
where
    Dir: OutputPin<Error = OutputPinError>,
{
    type WithDirectionControl =
        DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>;

    fn enable_direction_control(self, dir: Dir) -> Self::WithDirectionControl {
        DRV8825 {
            enable: self.enable,
            fault: self.fault,
            sleep: self.sleep,
            reset: self.reset,
            mode0: self.mode0,
            mode1: self.mode1,
            mode2: self.mode2,
            step: self.step,
            dir,
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> SetDirection
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Dir: OutputPin<Error = OutputPinError>,
{
    // 7.6 Timing Requirements (page 7)
    // https://www.ti.com/lit/ds/symlink/drv8825.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds::from_ticks(650);

    type Dir = Dir;
    type Error = Infallible;

    fn dir(&mut self) -> Result<&mut Self::Dir, Self::Error> {
        Ok(&mut self.dir)
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError>
    EnableStepControl<Step>
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, (), Dir>
where
    Step: OutputPin<Error = OutputPinError>,
{
    type WithStepControl =
        DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>;

    fn enable_step_control(self, step: Step) -> Self::WithStepControl {
        DRV8825 {
            enable: self.enable,
            fault: self.fault,
            sleep: self.sleep,
            reset: self.reset,
            mode0: self.mode0,
            mode1: self.mode1,
            mode2: self.mode2,
            step,
            dir: self.dir,
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> StepTrait
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Step: OutputPin<Error = OutputPinError>,
{
    // 7.6 Timing Requirements (page 7)
    // https://www.ti.com/lit/ds/symlink/drv8825.pdf
    const PULSE_LENGTH: Nanoseconds = Nanoseconds::from_ticks(1900);

    type Step = Step;
    type Error = Infallible;

    fn step(&mut self) -> Result<&mut Self::Step, Self::Error> {
        Ok(&mut self.step)
    }
}
