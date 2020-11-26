//! A4988 Driver
//!
//! Platform-agnostic driver for the A4988 stepper motor driver. This module
//! can be used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! The entry point to this module is the [`A4988`] struct.
//!
//! # Example
//!
//! ``` rust
//! # fn main()
//! #     -> Result<
//! #         (),
//! #         step_dir::a4988::StepError<core::convert::Infallible>
//! #     > {
//! #
//! use step_dir::{
//!     embedded_time::{duration::Microseconds, Clock as _},
//!     a4988::A4988,
//!     Dir, Step as _,
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
//! let mut driver = A4988::from_step_dir_pins(step, dir);
//!
//! // Rotate stepper motor by a few steps.
//! for _ in 0 .. 5 {
//!     let timer = clock.new_timer(STEP_DELAY).start()?;
//!     driver.step(Dir::Forward, &clock)?;
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
use embedded_time::{duration::Nanoseconds, Clock, TimeError};

use crate::{Dir as Direction, SetStepMode, Step as StepTrait, StepMode16};

/// The A4988 driver API
///
/// You can create an instance of this struct by calling
/// [`A4988::from_step_dir_pins`]. See [module documentation] for a full
/// example that uses this API.
///
/// [module documentation]: index.html
pub struct A4988<Enable, Sleep, Reset, MS1, MS2, MS3, Step, Dir> {
    enable: Enable,
    sleep: Sleep,
    reset: Reset,
    ms1: MS1,
    ms2: MS2,
    ms3: MS3,
    step: Step,
    dir: Dir,
}

impl<Step, Dir> A4988<(), (), (), (), (), (), Step, Dir> {
    /// Create a new instance of `A4988`
    ///
    /// Creates an instance of this struct from just the STEP and DIR pins. It
    /// expects the types that represent those pins to implement [`OutputPin`].
    ///
    /// The resulting instance can be used to step the motor using
    /// [`A4988::step`]. All other capabilities of the A4988, like
    /// the power-up sequence, selecting a step mode, or controlling the power
    /// state, explicitly enabled, or managed externally.
    ///
    /// To enable additional capabilities, see
    /// [`A4988::enable_mode_control`].
    pub fn from_step_dir_pins<Error>(step: Step, dir: Dir) -> Self
    where
        Step: OutputPin<Error = Error>,
        Dir: OutputPin<Error = Error>,
    {
        Self {
            enable: (),
            sleep: (),
            reset: (),
            ms1: (),
            ms2: (),
            ms3: (),
            step,
            dir,
        }
    }
}

impl<Step, Dir> A4988<(), (), (), (), (), (), Step, Dir> {
    /// Enables support for step mode control and sets the initial step mode
    ///
    /// Consumes this instance of `A4988` and returns another instance that
    /// has support for controlling the step mode. Requires the additional pins
    /// for doing so, namely RESET, MS1, MS2, and MS3. It expects the
    /// types that represent those pins to implement [`OutputPin`].
    ///
    /// This method is only available when those pins have not been provided
    /// yet. After this method has been called once, you can use
    /// [`A4988::set_step_mode`] to change the step mode again.
    pub fn enable_mode_control<Reset, MS1, MS2, MS3, Clk, OutputPinError>(
        self,
        reset: Reset,
        ms1: MS1,
        ms2: MS2,
        ms3: MS3,
        step_mode: StepMode16,
        clock: &Clk,
    ) -> Result<
        A4988<(), (), Reset, MS1, MS2, MS3, Step, Dir>,
        ModeError<OutputPinError>,
    >
    where
        Reset: OutputPin<Error = OutputPinError>,
        MS1: OutputPin<Error = OutputPinError>,
        MS2: OutputPin<Error = OutputPinError>,
        MS3: OutputPin<Error = OutputPinError>,
        Clk: Clock,
    {
        let mut self_ = A4988 {
            enable: self.enable,
            sleep: self.sleep,
            reset,
            ms1,
            ms2,
            ms3,
            step: self.step,
            dir: self.dir,
        };

        self_.set_step_mode(step_mode, clock)?;

        Ok(self_)
    }
}

impl<Reset, MS1, MS2, MS3, Step, Dir, OutputPinError> SetStepMode
    for A4988<(), (), Reset, MS1, MS2, MS3, Step, Dir>
where
    Reset: OutputPin<Error = OutputPinError>,
    MS1: OutputPin<Error = OutputPinError>,
    MS2: OutputPin<Error = OutputPinError>,
    MS3: OutputPin<Error = OutputPinError>,
{
    type Error = ModeError<OutputPinError>;
    type StepMode = StepMode16;

    /// Sets the step mode
    ///
    /// This method is only available, if all the pins required for setting the
    /// step mode have been provided using [`A4988::enable_mode_control`].
    fn set_step_mode<Clk: Clock>(
        &mut self,
        step_mode: StepMode16,
        clock: &Clk,
    ) -> Result<(), Self::Error> {
        // Figure 1: Logic Interface Timing Diagram
        // https://www.allegromicro.com/-/media/files/datasheets/a4988-datasheet.ashx
        const SETUP_TIME: Nanoseconds = Nanoseconds(200);

        // Reset the device's internal logic and disable the h-bridge drivers.
        self.reset
            .try_set_low()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Set mode signals.
        let (ms1, ms2, ms3) = step_mode_to_signals(&step_mode);
        self.ms1
            .try_set_state(ms1)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.ms2
            .try_set_state(ms2)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.ms3
            .try_set_state(ms3)
            .map_err(|err| ModeError::OutputPin(err))?;

        // Need to wait for the MSx input setup time.
        clock.new_timer(SETUP_TIME).start()?.wait()?;

        Ok(())
    }
}

impl<Reset, MS1, MS2, MS3, Step, Dir, OutputPinError> StepTrait
    for A4988<(), (), Reset, MS1, MS2, MS3, Step, Dir>
where
    Step: OutputPin<Error = OutputPinError>,
    Dir: OutputPin<Error = OutputPinError>,
{
    type Error = StepError<OutputPinError>;

    /// Rotates the motor one (micro-)step in the given direction
    ///
    /// Sets the DIR pin according to the `dir` argument, initiates a step pulse
    /// by setting STEP HIGH, then ends the step pulse by setting STEP LOW
    /// again. The method blocks while this is going on.
    ///
    /// This should result in the motor making one step. To achieve a specific
    /// speed, the user must call this method at the appropriate frequency.
    ///
    /// Requires a reference to an `embedded_time::Clock` implementation to
    /// handle the timing. Please make sure that the timer doesn't overflow
    /// while this method is running.
    ///
    /// Any errors that occur are wrapped in a [`StepError`] and returned to the
    /// user directly. This might leave the driver API in an invalid state, for
    /// example if STEP has been set HIGH, but an error occurs before it can be
    /// set LOW again.
    fn step<Clk: Clock>(
        &mut self,
        dir: Direction,
        clock: &Clk,
    ) -> Result<(), Self::Error> {
        // Figure 1: Logic Interface Timing Diagram
        // https://www.allegromicro.com/-/media/files/datasheets/a4988-datasheet.ashx
        const SETUP_TIME: Nanoseconds = Nanoseconds(200);
        const PULSE_LENGTH: Nanoseconds = Nanoseconds(200);

        match dir {
            Direction::Forward => self
                .dir
                .try_set_high()
                .map_err(|err| StepError::OutputPin(err))?,
            Direction::Backward => self
                .dir
                .try_set_low()
                .map_err(|err| StepError::OutputPin(err))?,
        }

        // According to the datasheet, we need to wait at least 650ns between
        // setting DIR and starting the STEP pulse
        clock.new_timer(SETUP_TIME).start()?.wait()?;

        // Start step pulse
        self.step
            .try_set_high()
            .map_err(|err| StepError::OutputPin(err))?;

        // There are two delays we need to adhere to:
        // - The minimum DIR hold time of 650ns
        // - The minimum STEP high time of 1.9us
        clock.new_timer(PULSE_LENGTH).start()?.wait()?;

        // End step pulse
        self.step
            .try_set_low()
            .map_err(|err| StepError::OutputPin(err))?;

        Ok(())
    }
}

/// An error that can occur while setting the microstepping mode
#[derive(Debug, Eq, PartialEq)]
pub enum ModeError<OutputPinError> {
    /// An error originated from using the [`OutputPin`] trait
    OutputPin(OutputPinError),

    /// An error originated from working with a timer
    Time(TimeError),
}

impl<OutputPinError> From<TimeError> for ModeError<OutputPinError> {
    fn from(err: TimeError) -> Self {
        Self::Time(err)
    }
}

/// An error that can occur while making a step
#[derive(Debug, Eq, PartialEq)]
pub enum StepError<OutputPinError> {
    /// An error originated from using the [`OutputPin`] trait
    OutputPin(OutputPinError),

    /// An error originated from working with a timer
    Time(TimeError),
}

impl<OutputPinError> From<TimeError> for StepError<OutputPinError> {
    fn from(err: TimeError) -> Self {
        Self::Time(err)
    }
}

/// Provides the pin signals for the given step mode
fn step_mode_to_signals(
    step_mode: &StepMode16,
) -> (PinState, PinState, PinState) {
    use PinState::*;
    use StepMode16::*;

    match step_mode {
        Full => (Low, Low, Low),
        M2 => (High, Low, Low),
        M4 => (Low, High, Low),
        M8 => (High, High, Low),
        M16 => (High, High, High),
    }
}
