//! DRV8825 Driver
//!
//! Platform-agnostic driver for the DRV8825 stepper motor driver. This module
//! can be used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! The entry point to this module is the [`DRV8825`] struct.
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
//!     drivers::drv8825::DRV8825,
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
//! # let step = Pin;
//! # let dir = Pin;
//! # let mut clock = Clock(std::time::Instant::now());
//! #
//! // You need to acquire the GPIO pins connected to the STEP and DIR signals.
//! // How you do this depends on your target platform. All the driver cares
//! // about is that they implement `embedded_hal::digital::OutputPin`. You also
//! // need an implementation of `embedded_hal::blocking::DelayUs`.
//!
//! // Create driver API from STEP and DIR pins.
//! let mut driver = Driver::from_inner(DRV8825::new())
//!     .enable_direction_control(dir, Direction::Forward, &clock)?
//!     .enable_step_control(step);
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
        SetDirection, SetStepMode, Step as StepTrait,
    },
    StepMode32,
};

/// The DRV8825 driver API
///
/// You can create an instance of this struct by calling [`DRV8825::new`]. See
/// [module documentation] for a full example that uses this API.
///
/// [module documentation]: index.html
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
    ///
    /// The resulting instance won't be able to do anything yet. You can call
    /// the various `enable_` methods of [`Driver`](crate::Driver) to rectify
    /// that.
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
    const SETUP_TIME: Nanoseconds = Nanoseconds(650);
    const HOLD_TIME: Nanoseconds = Nanoseconds(650);

    type Error = OutputPinError;
    type StepMode = StepMode32;

    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error> {
        // Reset the device's internal logic and disable the h-bridge drivers.
        self.reset.try_set_low()?;

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
        self.mode0.try_set_state(mode0)?;
        self.mode1.try_set_state(mode1)?;
        self.mode2.try_set_state(mode2)?;

        Ok(())
    }

    fn enable_driver(&mut self) -> Result<(), Self::Error> {
        self.reset.try_set_high()
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
    const SETUP_TIME: Nanoseconds = Nanoseconds(650);

    type Dir = Dir;
    type Error = OutputPinError;

    fn dir(&mut self) -> &mut Self::Dir {
        &mut self.dir
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
    const PULSE_LENGTH: Nanoseconds = Nanoseconds(1900);

    type Step = Step;
    type Error = OutputPinError;

    fn step(&mut self) -> &mut Self::Step {
        &mut self.step
    }
}
