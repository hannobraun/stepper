//! Compatibility code to help use Stepper on more platforms

use core::convert::{Infallible, TryFrom};

use embedded_hal::{digital::OutputPin, timer::CountDown};
use embedded_hal_stable::{
    digital::v2::OutputPin as StableOutputPin,
    timer::CountDown as StableCountDown,
};
use embedded_time::{
    duration::Duration as _, rate::Fraction, ConversionError, TimeInt,
};

/// Wrapper around a pin
///
/// Provides an implementation of [`embedded_hal::digital::OutputPin`] (that is,
/// the `OutputPin` from the latest alpha version of `embedded-hal`) for all
/// types that implement `OutputPin` from the latest stable version of
/// `embedded-hal`.
pub struct Pin<T>(pub T);

impl<T> OutputPin for Pin<T>
where
    T: StableOutputPin,
{
    type Error = <T as StableOutputPin>::Error;

    fn try_set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }

    fn try_set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }
}

/// Wrapper around a timer
///
/// Provides an implementation of [`embedded_hal::timer::CountDown`] (that is,
/// the `CountDown` from the latest alpha version of `embedded-hal`) for all
/// types that implement `CountDown` from the latest stable version of
/// `embedded-hal`.
pub struct Timer<T, const FREQ: u32>(pub T);

impl<T, const FREQ: u32> CountDown for Timer<T, FREQ>
where
    T: StableCountDown,
{
    type Error = Infallible;

    type Time = Ticks<<T as StableCountDown>::Time, FREQ>;

    fn try_start<Ticks>(&mut self, ticks: Ticks) -> Result<(), Self::Error>
    where
        Ticks: Into<Self::Time>,
    {
        let ticks = ticks.into();
        self.0.start(ticks.0);
        Ok(())
    }

    fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
        match self.0.wait() {
            Ok(()) => Ok(()),
            Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(_)) => {
                unreachable!("Caught error from infallible method")
            }
        }
    }
}

/// Timer ticks for a timer with frequency `FREQ`
///
/// Provides conversions from various duration types from `embedded-time` into
/// timer ticks for a timer with the frequency defined by `FREQ`. Since this is
/// a fully generic type that has no knowledge of the timers it is being used
/// with, it is the user's responsibility to make sure the resulting value is
/// valid for the timer.
///
/// `FREQ` is defined in Hz.
pub struct Ticks<T, const FREQ: u32>(pub T);

macro_rules! impl_conversions {
    ($($duration:ident,)*) => {
        $(
            impl<T, const FREQ: u32> TryFrom<embedded_time::duration::$duration>
                for Ticks<T, FREQ>
            where
                T: TimeInt,
            {
                type Error = ConversionError;

                fn try_from(duration: embedded_time::duration::$duration)
                    -> Result<Self, Self::Error>
                {
                    let ticks =
                        duration.to_generic::<T>(Fraction::new(1, FREQ))?;
                    Ok(Self(ticks.integer()))
                }
            }
        )*
    };
}

impl_conversions!(
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
);
