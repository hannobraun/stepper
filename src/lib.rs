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
//! # fn main() -> Result<(), core::convert::Infallible> {
//! use embedded_hal::prelude::*;
//! use stspin220::{Dir, STEP_METHOD_DELAY_US, STSPIN220};
//!
//! const STEP_DELAY_US: u32 = 500; // defines the motor speed
//!
//! # struct Pin;
//! # impl embedded_hal::digital::OutputPin for Pin {
//! #     type Error = core::convert::Infallible;
//! #     fn try_set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn try_set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! #
//! # struct Delay;
//! # impl embedded_hal::blocking::delay::DelayUs<u32> for Delay {
//! #     type Error = core::convert::Infallible;
//! #     fn try_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
//! #         Ok(())
//! #     }
//! # }
//! #
//! # let step_mode3 = Pin;
//! # let dir_mode4 = Pin;
//! # let mut delay = Delay;
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
//!     driver.step(Dir::Forward, &mut delay)?;
//!     delay.try_delay_us(STEP_DELAY_US - STEP_METHOD_DELAY_US as u32)?;
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

use embedded_hal::{blocking::delay::DelayUs, digital::OutputPin};

const DIR_SETUP_DELAY_US: u8 = 1;
const PULSE_LENGTH_US: u8 = 1;

/// The approximate duration that `STSPIN220::step` blocks for
///
/// Can be used by the user to calculate the time until the next call to
/// [`STSPIN220::step`].
pub const STEP_METHOD_DELAY_US: u8 = DIR_SETUP_DELAY_US + PULSE_LENGTH_US;

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
    _standby_reset: StandbyReset,
    _mode1: Mode1,
    _mode2: Mode2,
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
            _standby_reset: (),
            _mode1: (),
            _mode2: (),
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
            _standby_reset: standby_reset,
            _mode1: mode1,
            _mode2: mode2,
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
        self._standby_reset
            .try_set_low()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Set mode signals. All this repetition is messy. I decided not to do
        // anything about it and wait for the next embedded-hal alpha version,
        // which has features that would help here.
        let (mode1_s, mode2_s, mode3_s, mode4_s) = step_mode.to_signals();
        match mode1_s {
            false => self._mode1.try_set_low(),
            true => self._mode1.try_set_high(),
        }
        .map_err(|err| ModeError::OutputPin(err))?;
        match mode2_s {
            false => self._mode2.try_set_low(),
            true => self._mode2.try_set_high(),
        }
        .map_err(|err| ModeError::OutputPin(err))?;
        match mode3_s {
            false => self.step_mode3.try_set_low(),
            true => self.step_mode3.try_set_high(),
        }
        .map_err(|err| ModeError::OutputPin(err))?;
        match mode4_s {
            false => self.dir_mode4.try_set_low(),
            true => self.dir_mode4.try_set_high(),
        }
        .map_err(|err| ModeError::OutputPin(err))?;

        // Need to wait for the MODEx input setup time.
        delay
            .try_delay_us(MODE_SETUP_TIME_US.into())
            .map_err(|err| ModeError::Delay(err))?;

        // Leave standby mode.
        self._standby_reset
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
    /// STEP/MODE4 LOW again. The method blocks while this is going on. The
    /// approximate duration this method blocks for is defined by
    /// `STEP_LENGTH_US`.
    ///
    /// This should result in the motor making one step. To achieve a specific
    /// speed, the user must call this method at the appropriate frequency.
    ///
    /// Any errors that occur are wrapped in a [`StepError`] and returned to the
    /// user directly. This might leave the driver API in an invalid state, for
    /// example if STEP/MODE3 has been set HIGH, but an error occurs before it
    /// can be set LOW again.
    pub fn step<Delay, DelayTime, OutputPinError, DelayError>(
        &mut self,
        dir: Dir,
        delay: &mut Delay,
    ) -> Result<(), StepError<OutputPinError, DelayError>>
    where
        StepMode3: OutputPin<Error = OutputPinError>,
        DirMode4: OutputPin<Error = OutputPinError>,
        Delay: DelayUs<DelayTime, Error = DelayError>,
        DelayTime: From<u8>,
    {
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
        // setting DIR and starting the STEP pulse. We can't get that much
        // accuracy out of `DelayUs`, but waiting for a microsecond should be
        // fine for now.
        delay
            .try_delay_us(DIR_SETUP_DELAY_US.into())
            .map_err(|err| StepError::Delay(err))?;

        // Start step pulse
        self.step_mode3
            .try_set_high()
            .map_err(|err| StepError::OutputPin(err))?;

        // There are two delays we need to adhere to:
        // - The minimum DIR hold time of 100 ns.
        // - The minimum STCK high time, also 100 ns.
        //
        // The minimum delay we can get out of `DelayUs` is more than both, so
        // while not ideal, the following should be fine.
        delay
            .try_delay_us(PULSE_LENGTH_US.into())
            .map_err(|err| StepError::Delay(err))?;

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
    pub fn to_signals(&self) -> (bool, bool, bool, bool) {
        use StepMode::*;
        match self {
            Full => (false, false, false, false),
            M2 => (true, false, true, false),
            M4 => (false, true, false, true),
            M8 => (true, true, true, false),
            M16 => (true, true, true, true),
            M32 => (false, true, false, false),
            M64 => (true, true, false, true),
            M128 => (true, false, false, false),
            M256 => (true, true, false, false),
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepError<OutputPinError, DelayError> {
    /// An error originated from using the [`OutputPin`] trait
    OutputPin(OutputPinError),

    /// An error originated from using the [`DelayUs`] trait
    Delay(DelayError),
}

// Enables use of `?` in the (probably quite common) case when all error types
// are infallible.
impl From<StepError<Infallible, Infallible>> for Infallible {
    fn from(_: StepError<Infallible, Infallible>) -> Self {
        unreachable!()
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
