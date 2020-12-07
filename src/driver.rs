use embedded_time::TimeError;

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
