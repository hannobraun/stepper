//! Compatibility code to help use Stepper on more platforms

use core::fmt;
use embedded_hal::digital::OutputPin;
use embedded_hal::digital::{Error, ErrorKind, ErrorType};
use embedded_hal_stable::digital::v2::OutputPin as StableOutputPin;

/// Wrapper around a pin
///
/// Provides an implementation of [`embedded_hal::digital::OutputPin`]
/// (that is, the `OutputPin` from the latest alpha version of `embedded-hal`)
/// for all types that implement `OutputPin` from the latest stable version of
/// `embedded-hal`.
pub struct Pin<T>(pub T);

/// Wrapper for error compatibility
#[derive(Debug)]
pub struct CompatError<T>(pub T);

impl<T> Error for CompatError<T>
    where T: fmt::Debug
{
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl<T> ErrorType for Pin<T>
where
    T: StableOutputPin,
    T::Error: fmt::Debug,
{
    type Error = CompatError<T::Error>;
}

impl<T> OutputPin for Pin<T>
where
    T: StableOutputPin,
    T::Error: fmt::Debug,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low().map_err(CompatError)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_low().map_err(CompatError)
    }
}
