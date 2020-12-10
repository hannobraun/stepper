use embedded_hal::digital::OutputPin as _;
use embedded_time::{Clock, TimeError};

use crate::{
    traits::{SetStepMode, Step},
    Direction,
};

/// Abstract interface to stepper motor drivers
///
/// Wraps a concrete driver and uses the traits that the concrete driver
/// implements to provide an abstract API.
pub struct Driver<T> {
    inner: T,
}

impl<T> Driver<T> {
    /// Create a new `Driver` instance from a concrete driver
    ///
    /// Since `Driver` only provides an abstract interface for _using_ a driver,
    /// not for initializing it, you have to initialize a concrete driver and
    /// pass it to this constructor.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Access a reference to the wrapped driver
    ///
    /// Can be used to access driver-specific functionality that can't be
    /// provided by `Driver`'s abstract interface.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Access a mutable reference to the wrapped driver
    ///
    /// Can be used to access driver-specific functionality that can't be
    /// provided by `Driver`'s abstract interface.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Release the wrapped driver
    ///
    /// Drops this instance of `Driver` and returns the wrapped driver.
    pub fn release(self) -> T {
        self.inner
    }

    /// Sets the step mode
    ///
    /// This method is only available, if the wrapped driver supports
    /// microstepping, and supports setting the step mode through software. Some
    /// driver might not support microstepping at all, or only allow setting the
    /// step mode by changing physical switches.
    pub fn set_step_mode<Clk: Clock>(
        &mut self,
        step_mode: T::StepMode,
        clock: &Clk,
    ) -> Result<(), T::Error>
    where
        T: SetStepMode,
    {
        self.inner.set_step_mode(step_mode, clock)
    }

    /// Rotates the motor one (micro-)step in the given direction
    ///
    /// Steps the motor one step in the given direction, according to current
    /// microstep configuration. To achieve a specific speed, the user must call
    /// this method at the appropriate frequency.
    ///
    /// Requires a reference to an `embedded_time::Clock` implementation to
    /// handle the timing. Please make sure that the timer doesn't overflow
    /// while this method is running.
    pub fn step<Clk: Clock>(
        &mut self,
        dir: Direction,
        clock: &Clk,
    ) -> Result<(), StepError<T::Error>>
    where
        T: Step,
    {
        match dir {
            Direction::Forward => self
                .inner
                .dir()
                .try_set_high()
                .map_err(|err| StepError::OutputPin(err))?,
            Direction::Backward => self
                .inner
                .dir()
                .try_set_low()
                .map_err(|err| StepError::OutputPin(err))?,
        }

        clock.new_timer(T::SETUP_TIME).start()?.wait()?;

        // Start step pulse
        self.inner
            .step()
            .try_set_high()
            .map_err(|err| StepError::OutputPin(err))?;

        clock.new_timer(T::PULSE_LENGTH).start()?.wait()?;

        // End step pulse
        self.inner
            .step()
            .try_set_low()
            .map_err(|err| StepError::OutputPin(err))?;

        Ok(())
    }
}

/// An error that can occur while setting the microstepping mode
#[derive(Debug, Eq, PartialEq)]
pub enum ModeError<OutputPinError> {
    /// An error originated from using the [`OutputPin`] trait
    ///
    /// [`OutputPin`]: embedded_hal::digital::OutputPin
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
    ///
    /// [`OutputPin`]: embedded_hal::digital::OutputPin
    OutputPin(OutputPinError),

    /// An error originated from working with a timer
    Time(TimeError),
}

impl<OutputPinError> From<TimeError> for StepError<OutputPinError> {
    fn from(err: TimeError) -> Self {
        Self::Time(err)
    }
}
