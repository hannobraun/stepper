use embedded_time::TimeError;

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
