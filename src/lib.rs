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

use core::convert::Infallible;

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
    _enable_fault: EnableFault,
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
    /// state, have to be done externally.
    pub fn from_step_dir_pins<Error>(
        step_mode3: StepMode3,
        dir_mode4: DirMode4,
    ) -> Self
    where
        StepMode3: OutputPin<Error = Error>,
        DirMode4: OutputPin<Error = Error>,
    {
        Self {
            _enable_fault: (),
            _standby_reset: (),
            _mode1: (),
            _mode2: (),
            step_mode3,
            dir_mode4,
        }
    }
}

impl<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
    STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
{
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
