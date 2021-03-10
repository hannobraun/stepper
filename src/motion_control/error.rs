use core::fmt;

use crate::traits::{SetDirection, Step};

/// An error that can occur while using [`SoftwareMotionControl`]
///
/// [`SoftwareMotionControl`]: super::SoftwareMotionControl
pub enum Error<Driver, TimerError, NanosecondsToTicksError, DelayToTicksError>
where
    Driver: SetDirection + Step,
{
    /// Error while setting direction
    SetDirection(
        crate::Error<
            <Driver as SetDirection>::Error,
            NanosecondsToTicksError,
            TimerError,
        >,
    ),

    /// Error while stepping the motor
    Step(
        crate::Error<
            <Driver as Step>::Error,
            NanosecondsToTicksError,
            TimerError,
        >,
    ),

    /// Error while converting between time formats
    TimeConversion(
        TimeConversionError<NanosecondsToTicksError, DelayToTicksError>,
    ),

    /// Error while waiting for a step to finish
    StepDelay(TimerError),
}

// Can't `#[derive(Debug)]`, as that can't generate the required trait bounds.
impl<Driver, TimerError, NanosecondsToTicksError, DelayToTicksError> fmt::Debug
    for Error<Driver, TimerError, NanosecondsToTicksError, DelayToTicksError>
where
    Driver: SetDirection + Step,
    <Driver as SetDirection>::Error: fmt::Debug,
    <Driver as Step>::Error: fmt::Debug,
    TimerError: fmt::Debug,
    NanosecondsToTicksError: fmt::Debug,
    DelayToTicksError: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SetDirection(err) => {
                write!(f, "SetDirection(")?;
                err.fmt(f)?;
                write!(f, ")")?;
            }
            Self::Step(err) => {
                write!(f, "Step(")?;
                err.fmt(f)?;
                write!(f, ")")?;
            }
            Self::TimeConversion(err) => {
                write!(f, "TimeConversion(")?;
                err.fmt(f)?;
                write!(f, ")")?;
            }
            Self::StepDelay(err) => {
                write!(f, "StepDelay(")?;
                err.fmt(f)?;
                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

/// An error occurred while converting between time formats
#[derive(Debug)]
pub enum TimeConversionError<NanosecondsToTicksError, DelayToTicksError> {
    /// Error converting from nanoseconds to timer ticks
    NanosecondsToTicks(NanosecondsToTicksError),

    /// Error converting from RampMaker delay value to timer ticks
    DelayToTicks(DelayToTicksError),
}

/// The software motion control was busy, or another generic error occurred
#[derive(Debug)]
pub enum BusyError<T> {
    /// The software motion control was busy
    ///
    /// This happens while a movement is going on, and the driver is not
    /// available.
    Busy,

    /// Another error has occurred
    Other(T),
}
