use core::{
    convert::{TryFrom, TryInto},
    fmt,
};

use embedded_hal::timer;
use embedded_time::duration::Nanoseconds;
use ramp_maker::MotionProfile;

use crate::traits::{SetDirection, Step};

/// An error that can occur while using [`SoftwareMotionControl`]
///
/// [`SoftwareMotionControl`]: super::SoftwareMotionControl
pub enum Error<Driver, Timer, Profile>
where
    Driver: SetDirection + Step,
    Timer: timer::CountDown,
    Profile: MotionProfile,
    Timer::Time: TryFrom<Nanoseconds>,
    Profile::Delay: TryInto<Nanoseconds>,
{
    /// Error while setting direction
    SetDirection(
        crate::Error<
            <Driver as SetDirection>::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    ),

    /// Error while stepping the motor
    Step(
        crate::Error<
            <Driver as Step>::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    ),

    /// Error while converting between time formats
    TimeConversion(TimeConversionError<Timer::Time, Profile::Delay>),

    /// Error while waiting for a step to finish
    StepDelay(Timer::Error),
}

// Can't `#[derive(Debug)]`, as that can't generate the required trait bounds.
impl<Driver, Timer, Profile> fmt::Debug for Error<Driver, Timer, Profile>
where
    Driver: SetDirection + Step,
    Timer: timer::CountDown,
    Profile: MotionProfile,
    Timer::Time: TryFrom<Nanoseconds>,
    Profile::Delay: TryInto<Nanoseconds>,
    <Driver as SetDirection>::Error: fmt::Debug,
    <Driver as Step>::Error: fmt::Debug,
    Timer::Error: fmt::Debug,
    Timer::Time: fmt::Debug,
    <Timer::Time as TryFrom<Nanoseconds>>::Error: fmt::Debug,
    Profile::Delay: fmt::Debug,
    <Profile::Delay as TryInto<Nanoseconds>>::Error: fmt::Debug,
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
pub enum TimeConversionError<
    Time: TryFrom<Nanoseconds>,
    Delay: TryInto<Nanoseconds>,
> {
    /// Error converting from nanoseconds to timer ticks
    ToTimerTime(Time::Error),

    /// Error converting from timer ticks to nanoseconds
    FromDelay(Delay::Error),
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
