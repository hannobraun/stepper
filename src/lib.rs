//! STSPIN220 Driver
//!
//! Platform-agnostic driver library for the STSPIN220 stepper driver. This
//! library can run on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! The entry point to this library is the [`STSPIN220`] struct.
//!
//! # Example
//!
//! ``` rust
//! # fn main() -> Result<(), stspin220::StepError<core::convert::Infallible>> {
//! #
//! use embedded_time::{duration::Microseconds, Clock as _};
//! use stspin220::{Dir, STSPIN220};
//!
//! const STEP_DELAY: Microseconds = Microseconds(500);
//!
//! # struct Pin;
//! # impl embedded_hal::digital::OutputPin for Pin {
//! #     type Error = core::convert::Infallible;
//! #     fn try_set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn try_set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! #
//! # struct Clock(std::time::Instant);
//! # impl embedded_time::Clock for Clock {
//! #     type T = u32;
//! #     const SCALING_FACTOR: embedded_time::fraction::Fraction =
//! #         embedded_time::fraction::Fraction::new(1, 1_000_000);
//! #     fn try_now(&self)
//! #         -> Result<
//! #             embedded_time::Instant<Self>,
//! #             embedded_time::clock::Error
//! #         >
//! #     {
//! #         Ok(
//! #             embedded_time::Instant::new(
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
//! let mut driver = STSPIN220::from_step_dir_pins(step_mode3, dir_mode4);
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

#![no_std]
#![deny(missing_docs)]

use core::convert::{Infallible, TryFrom};

use embedded_hal::{
    blocking::delay::DelayUs,
    digital::{OutputPin, PinState},
};
use embedded_time::{duration::Nanoseconds, Clock, TimeError};

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

impl<StepMode3, DirMode4> STSPIN220<(), (), (), (), StepMode3, DirMode4> {
    /// Create a new instance of `STSPIN220`
    ///
    /// Creates an instance of this struct from just the STEP/MODE3 and
    /// DIR/MODE4 pins. It expects the types that represent those pins to
    /// implement [`OutputPin`].
    ///
    /// The resulting instance can be used to step the motor using
    /// [`STSPIN220::step`]. All other capabilities of the STSPIN220, like
    /// the power-up sequence, selecting a step mode, or controlling the power
    /// state, explicitly enabled, or managed externally.
    ///
    /// To enable additional capabilities, see
    /// [`STSPIN220::enable_mode_control`].
    pub fn from_step_dir_pins<Error>(
        step_mode3: StepMode3,
        dir_mode4: DirMode4,
    ) -> Self
    where
        StepMode3: OutputPin<Error = Error>,
        DirMode4: OutputPin<Error = Error>,
    {
        Self {
            enable_fault: (),
            standby_reset: (),
            mode1: (),
            mode2: (),
            step_mode3,
            dir_mode4,
        }
    }
}

impl<EnableFault, StepMode3, DirMode4>
    STSPIN220<EnableFault, (), (), (), StepMode3, DirMode4>
{
    /// Enables support for step mode control and sets the initial step mode
    ///
    /// Consumes this instance of `STSPIN220` and returns another instance that
    /// has support for controlling the step mode. Requires the additional pins
    /// for doing so, namely STBY/RESET, MODE1, and MODE2. It expects the types
    /// that represent those pins to implement [`OutputPin`].
    ///
    /// This method is only available when those pins have not been provided
    /// yet. After this method has been called once, you can use
    /// [`STSPIN::set_set_mode`] to change the step mode again.
    pub fn enable_mode_control<
        StandbyReset,
        Mode1,
        Mode2,
        Delay,
        DelayTime,
        OutputPinError,
        DelayError,
    >(
        self,
        standby_reset: StandbyReset,
        mode1: Mode1,
        mode2: Mode2,
        step_mode: StepMode,
        delay: &mut Delay,
    ) -> Result<
        STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>,
        ModeError<OutputPinError, DelayError>,
    >
    where
        StandbyReset: OutputPin<Error = OutputPinError>,
        Mode1: OutputPin<Error = OutputPinError>,
        Mode2: OutputPin<Error = OutputPinError>,
        StepMode3: OutputPin<Error = OutputPinError>,
        DirMode4: OutputPin<Error = OutputPinError>,
        Delay: DelayUs<DelayTime, Error = DelayError>,
        DelayTime: From<u8>,
    {
        let mut self_ = STSPIN220 {
            enable_fault: self.enable_fault,
            standby_reset,
            mode1,
            mode2,
            step_mode3: self.step_mode3,
            dir_mode4: self.dir_mode4,
        };

        self_.set_step_mode(step_mode, delay)?;

        Ok(self_)
    }
}

impl<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
    STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
{
    /// Sets the step mode
    ///
    /// This method is only available, if all the pins required for setting the
    /// step mode have been provided using [`STSPIN220::enable_mode_control`].
    pub fn set_step_mode<Delay, DelayTime, OutputPinError, DelayError>(
        &mut self,
        step_mode: StepMode,
        delay: &mut Delay,
    ) -> Result<(), ModeError<OutputPinError, DelayError>>
    where
        StandbyReset: OutputPin<Error = OutputPinError>,
        Mode1: OutputPin<Error = OutputPinError>,
        Mode2: OutputPin<Error = OutputPinError>,
        StepMode3: OutputPin<Error = OutputPinError>,
        DirMode4: OutputPin<Error = OutputPinError>,
        Delay: DelayUs<DelayTime, Error = DelayError>,
        DelayTime: From<u8>,
    {
        const MODE_SETUP_TIME_US: u8 = 1;
        const MODE_HOLD_TIME_US: u8 = 100;

        // Force driver into standby mode.
        self.standby_reset
            .try_set_low()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Set mode signals. All this repetition is messy. I decided not to do
        // anything about it and wait for the next embedded-hal alpha version,
        // which has features that would help here.
        let (mode1, mode2, mode3, mode4) = step_mode.to_signals();
        self.mode1
            .try_set_state(mode1)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.mode2
            .try_set_state(mode2)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.step_mode3
            .try_set_state(mode3)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.dir_mode4
            .try_set_state(mode4)
            .map_err(|err| ModeError::OutputPin(err))?;

        // Need to wait for the MODEx input setup time.
        delay
            .try_delay_us(MODE_SETUP_TIME_US.into())
            .map_err(|err| ModeError::Delay(err))?;

        // Leave standby mode.
        self.standby_reset
            .try_set_high()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Now the mode pins need to stay as they are for the MODEx input hold
        // time, for the settings to take effect.
        delay
            .try_delay_us(MODE_HOLD_TIME_US.into())
            .map_err(|err| ModeError::Delay(err))?;

        Ok(())
    }

    /// Rotates the motor one (micro-)step in the given direction
    ///
    /// Sets the DIR/MODE4 pin according to the `dir` argument, initiates a step
    /// pulse by setting STEP/MODE3 HIGH, then ends the step pulse by setting
    /// STEP/MODE4 LOW again. The method blocks while this is going on.
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
    /// example if STEP/MODE3 has been set HIGH, but an error occurs before it
    /// can be set LOW again.
    pub fn step<Clk, OutputPinError>(
        &mut self,
        dir: Dir,
        clock: &Clk,
    ) -> Result<(), StepError<OutputPinError>>
    where
        StepMode3: OutputPin<Error = OutputPinError>,
        DirMode4: OutputPin<Error = OutputPinError>,
        Clk: Clock,
    {
        const DIR_SETUP_DELAY: Nanoseconds = Nanoseconds(100);
        const PULSE_LENGTH: Nanoseconds = Nanoseconds(100);

        match dir {
            Dir::Forward => self
                .dir_mode4
                .try_set_high()
                .map_err(|err| StepError::OutputPin(err))?,
            Dir::Backward => self
                .dir_mode4
                .try_set_low()
                .map_err(|err| StepError::OutputPin(err))?,
        }

        // According to the datasheet, we need to wait at least 100 ns between
        // setting DIR and starting the STEP pulse.
        clock.new_timer(DIR_SETUP_DELAY).start()?.wait()?;

        // Start step pulse
        self.step_mode3
            .try_set_high()
            .map_err(|err| StepError::OutputPin(err))?;

        // There are two delays we need to adhere to:
        // - The minimum DIR hold time of 100 ns.
        // - The minimum STCK high time, also 100 ns.
        clock.new_timer(PULSE_LENGTH).start()?.wait()?;

        // End step pulse
        self.step_mode3
            .try_set_low()
            .map_err(|err| StepError::OutputPin(err))?;

        Ok(())
    }
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

/// An error that can occur while setting the microstepping mode
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeError<OutputPinError, DelayError> {
    /// An error originated from using the [`OutputPin`] trait
    OutputPin(OutputPinError),

    /// An error originated from using the [`DelayUs`] trait
    Delay(DelayError),
}

// Enables use of `?` in the (probably quite common) case when all error types
// are infallible.
impl From<ModeError<Infallible, Infallible>> for Infallible {
    fn from(_: ModeError<Infallible, Infallible>) -> Self {
        unreachable!()
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
