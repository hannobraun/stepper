/// Converts delay values from RampMaker into timer ticks
///
/// RampMaker is agnostic over the units used, and the unit of the timer ticks
/// depend on the target platform. This trait allows Step/Dir to convert between
/// both types. The user must supply an implementation that matches their
/// environment.
///
/// The `Delay` parameter specifies the type of delay value used by RampMaker.
pub trait DelayToTicks<Delay> {
    /// The timer ticks the delay is being converted into
    type Ticks;

    /// The error that can happen during conversion
    type Error;

    /// Convert delay value into timer ticks
    fn delay_to_ticks(&self, delay: Delay) -> Result<Self::Ticks, Self::Error>;
}
