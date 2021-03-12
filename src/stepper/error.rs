use crate::motion_control;

/// Unified error type
///
/// Unifies the two types of errors that can happen while using [`Stepper`]:
/// Signal errors and motion control errors.
///
/// [`Stepper`]: crate::Stepper
#[derive(Debug, Eq, PartialEq)]
pub enum Error<
    PinUnavailableError,
    PinError,
    NanosecondsToTicksError,
    DelayToTicksError,
    TimerError,
> {
    /// A signal error
    Signal(
        SignalError<
            PinUnavailableError,
            PinError,
            NanosecondsToTicksError,
            TimerError,
        >,
    ),

    /// A motion control error
    MotionControl(
        motion_control::Error<
            PinUnavailableError,
            PinError,
            PinUnavailableError,
            PinError,
            TimerError,
            NanosecondsToTicksError,
            DelayToTicksError,
        >,
    ),
}

impl<
        PinUnavailableError,
        PinError,
        NanosecondsToTicksError,
        DelayToTicksError,
        TimerError,
    >
    From<
        SignalError<
            PinUnavailableError,
            PinError,
            NanosecondsToTicksError,
            TimerError,
        >,
    >
    for Error<
        PinUnavailableError,
        PinError,
        NanosecondsToTicksError,
        DelayToTicksError,
        TimerError,
    >
{
    fn from(
        err: SignalError<
            PinUnavailableError,
            PinError,
            NanosecondsToTicksError,
            TimerError,
        >,
    ) -> Self {
        Self::Signal(err)
    }
}

impl<
        PinUnavailableError,
        PinError,
        NanosecondsToTicksError,
        DelayToTicksError,
        TimerError,
    >
    From<
        motion_control::Error<
            PinUnavailableError,
            PinError,
            PinUnavailableError,
            PinError,
            TimerError,
            NanosecondsToTicksError,
            DelayToTicksError,
        >,
    >
    for Error<
        PinUnavailableError,
        PinError,
        NanosecondsToTicksError,
        DelayToTicksError,
        TimerError,
    >
{
    fn from(
        err: motion_control::Error<
            PinUnavailableError,
            PinError,
            PinUnavailableError,
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
pub enum SignalError<
    PinUnavailableError,
    PinError,
    NanosecondsToTicksError,
    TimerError,
> {
    /// A pin was not accessible
    PinUnavailable(PinUnavailableError),

    /// An error originated from using the [`OutputPin`] trait
    ///
    /// [`OutputPin`]: embedded_hal::digital::OutputPin
    Pin(PinError),

    /// An error occurred while converting nanoseconds to timer ticks
    NanosecondsToTicks(NanosecondsToTicksError),

    /// An error originated from working with a timer
    Timer(TimerError),
}
