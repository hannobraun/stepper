//! Compatibility code to help use Stepper on more platforms

use core::fmt;

use embedded_hal::digital::ErrorType;
use embedded_hal::digital::OutputPin;
use embedded_hal_stable::digital::v2::OutputPin as StableOutputPin;

/// Wrapper around a pin
///
/// Provides an implementation of [`embedded_hal::digital::OutputPin`]
/// (that is, the `OutputPin` from the latest alpha version of `embedded-hal`)
/// for all types that implement `OutputPin` from the latest stable version of
/// `embedded-hal`.
pub struct Pin<T>(pub T);

impl<T> ErrorType for Pin<T>
where
    T: StableOutputPin,
    T::Error: fmt::Debug + embedded_hal::digital::Error,
{
    type Error = T::Error;
}

impl<T> OutputPin for Pin<T>
where
    T: StableOutputPin,
    T::Error: fmt::Debug + embedded_hal::digital::Error,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }
}
