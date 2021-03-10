use crate::motion_control;

/// Unified error type
///
/// Unifies the two types of errors that can happen while using [`Stepper`]:
/// Signal errors and motion control errors.
///
/// [`Stepper`]: crate::Stepper
#[derive(Debug, Eq, PartialEq)]
pub enum Error<PinError, NanosecondsToTicksError, DelayToTicksError, TimerError>
{
    /// A signal error
    Signal(SignalError<PinError, NanosecondsToTicksError, TimerError>),

    /// A motion control error
    MotionControl(
        motion_control::Error<
            PinError,
            PinError,
            TimerError,
            NanosecondsToTicksError,
            DelayToTicksError,
        >,
    ),
}

impl<PinError, NanosecondsToTicksError, DelayToTicksError, TimerError>
    From<SignalError<PinError, NanosecondsToTicksError, TimerError>>
    for Error<PinError, NanosecondsToTicksError, DelayToTicksError, TimerError>
{
    fn from(
        err: SignalError<PinError, NanosecondsToTicksError, TimerError>,
    ) -> Self {
        Self::Signal(err)
    }
}

impl<PinError, NanosecondsToTicksError, DelayToTicksError, TimerError>
    From<
        motion_control::Error<
            PinError,
            PinError,
            TimerError,
            NanosecondsToTicksError,
            DelayToTicksError,
        >,
    >
    for Error<PinError, NanosecondsToTicksError, DelayToTicksError, TimerError>
{
    fn from(
        err: motion_control::Error<
            PinError,
            PinError,
            TimerError,
            NanosecondsToTicksError,
            DelayToTicksError,
        >,
    ) -> Self {
        Self::MotionControl(err)
    }
}

/// An error that can occur while using this API
#[derive(Debug, Eq, PartialEq)]
pub enum SignalError<PinError, NanosecondsToTicksError, TimerError> {
    /// An error originated from using the [`OutputPin`] trait
    ///
    /// [`OutputPin`]: embedded_hal::digital::OutputPin
    Pin(PinError),

    /// An error occurred while converting nanoseconds to timer ticks
    NanosecondsToTicks(NanosecondsToTicksError),

    /// An error originated from working with a timer
    Timer(TimerError),
}
