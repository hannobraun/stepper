use embedded_hal::digital::OutputPin as _;
use embedded_time::{Clock, TimeError};

use crate::{
    traits::{
        Dir, EnableDirectionControl, EnableStepControl, EnableStepModeControl,
        SetStepMode, Step,
    },
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

    /// Enable microstepping mode control
    ///
    /// Consumes `Driver` and returns a new instance that provides control over
    /// the microstepping mode. Once this method has been called, the
    /// [`Driver::set_step_mode`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// microstepping mode as an argument. What exactly those are depends on the
    /// specific driver. Typically they are the output pins that are connected
    /// to the mode pins of the driver.
    ///
    /// This method is only available, if the driver supports enabling step mode
    /// control. It might no longer be available, once step mode control has
    /// been enabled.
    pub fn enable_step_mode_control<Resources>(
        self,
        res: Resources,
    ) -> Driver<T::WithStepModeControl>
    where
        T: EnableStepModeControl<Resources>,
    {
        Driver {
            inner: self.inner.enable_step_mode_control(res),
        }
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

    /// Enable direction control
    ///
    /// Consumes `Driver` and returns a new instance that provides control over
    /// the motor direction. Once this method has been called, the
    /// [`Driver::set_direction`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// direction as an argument. What exactly those are depends on the specific
    /// driver. Typically it's going to be the output pin that is connected to
    /// the driver's DIR pin.
    ///
    /// This method is only available, if the driver supports enabling direction
    /// control. It might no longer be available, once direction control has
    /// been enabled.
    pub fn enable_direction_control<Resources>(
        self,
        res: Resources,
    ) -> Driver<T::WithDirectionControl>
    where
        T: EnableDirectionControl<Resources>,
    {
        Driver {
            inner: self.inner.enable_direction_control(res),
        }
    }

    /// Set direction for future movements
    ///
    /// Requires a reference to an `embedded_time::Clock` implementation to
    /// handle the timing. Please make sure that the timer doesn't overflow
    /// while this method is running.
    pub fn set_direction<Clk: Clock>(
        &mut self,
        direction: Direction,
        clock: &Clk,
    ) -> Result<(), StepError<T::Error>>
    where
        T: Dir,
    {
        match direction {
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

        Ok(())
    }

    /// Enable step control
    ///
    /// Consumes `Driver` and returns a new instance that provides control over
    /// stepping the motor. Once this method has been called, the
    /// [`Driver::step`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// direction as an argument. What exactly those are depends on the specific
    /// driver. Typically it's going to be the output pin that is connected to
    /// the driver's STEP pin.
    ///
    /// This method is only available, if the driver supports enabling step
    /// control. It might no longer be available, once step control has been
    /// enabled.
    pub fn enable_step_control<Resources>(
        self,
        res: Resources,
    ) -> Driver<T::WithStepControl>
    where
        T: EnableStepControl<Resources>,
    {
        Driver {
            inner: self.inner.enable_step_control(res),
        }
    }

    /// Rotates the motor one (micro-)step in the given direction
    ///
    /// Steps the motor one step in the direction that was previously set,
    /// according to current microstep configuration. To achieve a specific
    /// speed, the user must call this method at the appropriate frequency.
    ///
    /// Requires a reference to an `embedded_time::Clock` implementation to
    /// handle the timing. Please make sure that the timer doesn't overflow
    /// while this method is running.
    pub fn step<Clk: Clock>(
        &mut self,
        clock: &Clk,
    ) -> Result<(), StepError<T::Error>>
    where
        T: Step,
    {
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
