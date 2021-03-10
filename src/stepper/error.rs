/// An error that can occur while using this API
#[derive(Debug, Eq, PartialEq)]
pub enum SignalError<PinError, TimeConversionError, TimerError> {
    /// An error originated from using the [`OutputPin`] trait
    ///
    /// [`OutputPin`]: embedded_hal::digital::OutputPin
    Pin(PinError),

    /// An error occurred while converting time to timer ticks
    TimeConversion(TimeConversionError),

    /// An error originated from working with a timer
    Timer(TimerError),
}
