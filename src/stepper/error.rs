use crate::motion_control;

/// Unified error type
///
/// Unifies the two types of errors that can happen while using [`Stepper`]:
/// Signal errors and motion control errors.
///
/// [`Stepper`]: crate::Stepper
#[derive(Debug, Eq, PartialEq)]
pub enum Error<PinUnavailableError, PinError, DelayToTicksError, TimerError> {
    /// A signal error
    Signal(SignalError<PinUnavailableError, PinError, TimerError>),

    /// A motion control error
    MotionControl(
        motion_control::Error<
            PinUnavailableError,
            PinError,
            PinUnavailableError,
            PinError,
            TimerError,
            DelayToTicksError,
        >,
    ),
}

impl<PinUnavailableError, PinError, DelayToTicksError, TimerError>
    From<SignalError<PinUnavailableError, PinError, TimerError>>
    for Error<PinUnavailableError, PinError, DelayToTicksError, TimerError>
{
    fn from(
        err: SignalError<PinUnavailableError, PinError, TimerError>,
    ) -> Self {
        Self::Signal(err)
    }
}

impl<PinUnavailableError, PinError, DelayToTicksError, TimerError>
    From<
        motion_control::Error<
            PinUnavailableError,
            PinError,
            PinUnavailableError,
            PinError,
            TimerError,
            DelayToTicksError,
        >,
    > for Error<PinUnavailableError, PinError, DelayToTicksError, TimerError>
{
    fn from(
        err: motion_control::Error<
            PinUnavailableError,
            PinError,
            PinUnavailableError,
            PinError,
            TimerError,
            DelayToTicksError,
        >,
    ) -> Self {
        Self::MotionControl(err)
    }
}

/// An error that can occur while using this API
#[derive(Debug, Eq, PartialEq)]
pub enum SignalError<PinUnavailableError, PinError, TimerError> {
    /// A pin was not accessible
    PinUnavailable(PinUnavailableError),

    /// An error originated from using the [`OutputPin`] trait
    ///
    /// [`OutputPin`]: embedded_hal::digital::OutputPin
    Pin(PinError),

    /// An error originated from working with a timer
    Timer(TimerError),
}
