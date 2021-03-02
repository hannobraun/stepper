//! Software implementation of motion control capability
//!
//! See [`SoftwareMotionControl`] for more information.

use core::{
    convert::{TryFrom, TryInto},
    fmt,
    task::Poll,
};

use embedded_hal::timer;
use embedded_time::duration::Nanoseconds;
use ramp_maker::MotionProfile;
use replace_with::replace_with_and_return;

use crate::{
    traits::{EnableMotionControl, MotionControl, SetDirection, Step},
    Direction, SetDirectionFuture, StepFuture,
};

/// Software implementation of motion control capability
///
/// Some driver natively support motion control capability. This is a software
/// implementation of the [`MotionControl`] trait for those drivers that don't.
/// It wraps a driver that implements [`SetDirection`] and [`Step`], and in turn
/// acts like a driver itself, adding to the wrapped driver's capabilities.
///
/// You can use `SoftwareMotionControl` directly, but like a driver, it is
/// designed to be used through the [`Stepper`] API.
///
/// [`Stepper`]: crate::Stepper
pub struct SoftwareMotionControl<Driver, Timer, Profile: MotionProfile> {
    state: State<Driver, Timer, Profile>,
    new_motion: Option<Direction>,
    profile: Profile,
    current_step: i32,
    current_direction: Direction,
}

impl<Driver, Timer, Profile> SoftwareMotionControl<Driver, Timer, Profile>
where
    Profile: MotionProfile,
{
    /// Construct a new instance of `SoftwareMotionControl`
    ///
    /// Instead of using this constructor directly, you can instead use
    /// [`Stepper::enable_motion_control`] with any driver that implements
    /// [`SetDirection`] and [`Step`], providing timer and a motion profile.
    /// This module provides a blanket implementation of [`EnableMotionControl`]
    /// to make this work.
    ///
    /// [`Stepper::enable_motion_control`]: crate::Stepper::enable_motion_control
    pub fn new(driver: Driver, timer: Timer, profile: Profile) -> Self {
        Self {
            state: State::Idle { driver, timer },
            new_motion: None,
            profile,
            current_step: 0,
            // Doesn't matter what we initialize it with. We're only using it
            // during an ongoing movement, and it will have been overridden at
            // that point.
            current_direction: Direction::Forward,
        }
    }

    /// Access a reference to the wrapped driver
    ///
    /// This is only possible if there is no ongoing movement.
    pub fn driver(&self) -> Option<&Driver> {
        if let State::Idle { driver, .. } = &self.state {
            return Some(driver);
        }

        None
    }

    /// Access a mutable reference to the wrapped driver
    ///
    /// This is only possible if there is no ongoing movement.
    pub fn driver_mut(&mut self) -> Option<&mut Driver> {
        if let State::Idle { driver, .. } = &mut self.state {
            return Some(driver);
        }

        None
    }

    /// Access a reference to the wrapped timer
    ///
    /// This is only possible if there is no ongoing movement.
    pub fn timer(&self) -> Option<&Timer> {
        if let State::Idle { timer, .. } = &self.state {
            return Some(timer);
        }

        None
    }

    /// Access a mutable reference to the wrapped timer
    ///
    /// This is only possible if there is no ongoing movement.
    pub fn timer_mut(&mut self) -> Option<&mut Timer> {
        if let State::Idle { timer, .. } = &mut self.state {
            return Some(timer);
        }

        None
    }

    /// Access a reference to the wrapped motion profile
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Access a mutable reference to the wrapped motion profile
    pub fn profile_mut(&mut self) -> &mut Profile {
        &mut self.profile
    }

    /// Access the current step
    pub fn current_step(&self) -> i32 {
        self.current_step
    }

    /// Access the current direction
    pub fn current_direction(&self) -> Direction {
        self.current_direction
    }
}

impl<Driver, Timer, Profile> MotionControl
    for SoftwareMotionControl<Driver, Timer, Profile>
where
    Driver: SetDirection + Step,
    Profile: MotionProfile,
    Timer: timer::CountDown,
    Timer::Time: TryFrom<Nanoseconds>,
    Profile::Delay: TryInto<Nanoseconds>,
    Profile::Velocity: Copy,
{
    type Velocity = Profile::Velocity;
    type Error = Error<Driver, Timer, Profile>;

    fn move_to_position(
        &mut self,
        max_velocity: Self::Velocity,
        target_step: i32,
    ) -> Result<(), Self::Error> {
        let steps_from_here = target_step - self.current_step;

        self.profile
            .enter_position_mode(max_velocity, steps_from_here.abs() as u32);

        let direction = if steps_from_here > 0 {
            Direction::Forward
        } else {
            Direction::Backward
        };
        self.new_motion = Some(direction);

        Ok(())
    }

    fn update(&mut self) -> Result<bool, Self::Error> {
        // Otherwise the closure will borrow all of `self`.
        let new_motion = &mut self.new_motion;
        let profile = &mut self.profile;
        let current_step = &mut self.current_step;
        let current_direction = &mut self.current_direction;

        replace_with_and_return(
            &mut self.state,
            || State::Invalid,
            |state| {
                update_state(
                    state,
                    new_motion,
                    profile,
                    current_step,
                    current_direction,
                )
            },
        )
    }
}

enum State<Driver, Timer, Profile: MotionProfile> {
    Idle {
        driver: Driver,
        timer: Timer,
    },
    SetDirection(SetDirectionFuture<Driver, Timer>),
    Step {
        future: StepFuture<Driver, Timer>,
        delay: Profile::Delay,
    },
    StepDelay {
        driver: Driver,
        timer: Timer,
    },
    Invalid,
}

/// An error that can occur while using [`SoftwareMotionControl`]
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

#[derive(Debug)]
/// An error occurred while converting between time formats
pub enum TimeConversionError<
    Time: TryFrom<Nanoseconds>,
    Delay: TryInto<Nanoseconds>,
> {
    /// Error converting from nanoseconds to timer ticks
    ToTimerTime(Time::Error),

    /// Error converting from timer ticks to nanoseconds
    FromDelay(Delay::Error),
}

fn update_state<Driver, Timer, Profile>(
    mut state: State<Driver, Timer, Profile>,
    new_motion: &mut Option<Direction>,
    profile: &mut Profile,
    current_step: &mut i32,
    current_direction: &mut Direction,
) -> (
    Result<bool, Error<Driver, Timer, Profile>>,
    State<Driver, Timer, Profile>,
)
where
    Driver: SetDirection + Step,
    Timer: timer::CountDown,
    Timer::Time: TryFrom<Nanoseconds>,
    Profile: MotionProfile,
    Profile::Delay: TryInto<Nanoseconds>,
{
    loop {
        match state {
            State::Idle { driver, timer } => {
                // Being idle can mean that there's actually nothing to do, or
                // it might just be a short breather before more work comes in.

                if let Some(direction) = new_motion.take() {
                    // A new motion has been started. This might override an
                    // ongoing one, but it makes no difference here.
                    //
                    // Let's update the state, but don't return just yet. We
                    // have more stuff to do (polling the future).
                    state = State::SetDirection(SetDirectionFuture::new(
                        direction, driver, timer,
                    ));
                    *current_direction = direction;
                    continue;
                }

                // No new motion has been started, but we might still have an
                // ongoing one. Let's ask the motion profile.
                if let Some(delay) = profile.next_delay() {
                    // There's a motion ongoing. Let's start the next step, but
                    // again, don't return yet. The future needs to be polled.
                    state = State::Step {
                        future: StepFuture::new(driver, timer),
                        delay,
                    };
                    continue;
                }

                // Now we know that there's truly nothing to do. Return to the
                // caller and stay idle.
                return (Ok(false), State::Idle { driver, timer });
            }
            State::SetDirection(mut future) => {
                match future.poll() {
                    Poll::Ready(Ok(())) => {
                        // Direction has been set. Set state back to idle, so we
                        // can figure out what to do next in the next loop
                        // iteration.
                        let (driver, timer) = future.release();
                        state = State::Idle { driver, timer };
                        continue;
                    }
                    Poll::Ready(Err(err)) => {
                        // Error happened while setting direction. We need to
                        // let the caller know.
                        //
                        // The state stays as it is. For all we know, the error
                        // can be recovered from.
                        return (
                            Err(Error::SetDirection(err)),
                            State::SetDirection(future),
                        );
                    }
                    Poll::Pending => {
                        // Still busy setting direction. Let caller know.
                        return (Ok(true), State::SetDirection(future));
                    }
                }
            }
            State::Step { mut future, delay } => {
                match future.poll() {
                    Poll::Ready(Ok(())) => {
                        // A step was made. Now we need to wait out the rest of
                        // the step delay before we can do something else.

                        *current_step += *current_direction as i32;

                        let (driver, mut timer) = future.release();
                        let delay_left: Timer::Time =
                            match delay_left(delay, Driver::PULSE_LENGTH) {
                                Ok(delay_left) => delay_left,
                                Err(err) => {
                                    return (
                                        Err(Error::TimeConversion(err)),
                                        State::Idle { driver, timer },
                                    )
                                }
                            };

                        if let Err(err) = timer.try_start(delay_left) {
                            return (
                                Err(Error::StepDelay(err)),
                                State::Idle { driver, timer },
                            );
                        }

                        state = State::StepDelay { driver, timer };
                        continue;
                    }
                    Poll::Ready(Err(err)) => {
                        // Error happened while stepping. Need to
                        // let the caller know.
                        //
                        // State stays as it is. For all we know,
                        // the error can be recovered from.
                        return (
                            Err(Error::Step(err)),
                            State::Step { future, delay },
                        );
                    }
                    Poll::Pending => {
                        // Still stepping. Let caller know.
                        return (Ok(true), State::Step { future, delay });
                    }
                }
            }
            State::StepDelay { driver, mut timer } => {
                match timer.try_wait() {
                    Ok(()) => {
                        // We've waited out the step delay. Return to idle
                        // state, to figure out what's next.
                        state = State::Idle { driver, timer };
                        continue;
                    }
                    Err(nb::Error::WouldBlock) => {
                        // The timer is still running. Let the user know.
                        return (Ok(true), State::StepDelay { driver, timer });
                    }
                    Err(nb::Error::Other(err)) => {
                        // Error while trying to wait. Need to tell the caller.
                        return (
                            Err(Error::StepDelay(err)),
                            State::StepDelay { driver, timer },
                        );
                    }
                }
            }
            State::Invalid => {
                // This can only happen if this closure panics, the
                // user catches the panic, then attempts to
                // continue.
                //
                // A panic in this closure is always going to be a
                // bug, and once that happened, we're in an invalid
                // state. Not a lot we can do about it.
                panic!("Invalid internal state, caused by a previous panic.")
            }
        }
    }
}

fn delay_left<Delay, Time>(
    delay: Delay,
    pulse_length: Nanoseconds,
) -> Result<Time, TimeConversionError<Time, Delay>>
where
    Time: TryFrom<Nanoseconds>,
    Delay: TryInto<Nanoseconds>,
{
    let delay: Nanoseconds = delay
        .try_into()
        .map_err(|err| TimeConversionError::FromDelay(err))?;
    let delay_left: Time = (delay - pulse_length)
        .try_into()
        .map_err(|err| TimeConversionError::ToTimerTime(err))?;
    Ok(delay_left)
}

// Blanket implementation of `EnableMotionControl` for all STEP/DIR stepper
// drivers.
impl<Driver, Timer, Profile> EnableMotionControl<(Timer, Profile)> for Driver
where
    Driver: SetDirection + Step,
    Profile: MotionProfile,
    Timer: timer::CountDown,
    Timer::Time: TryFrom<Nanoseconds>,
    Profile::Delay: TryInto<Nanoseconds>,
    Profile::Velocity: Copy,
{
    type WithMotionControl = SoftwareMotionControl<Driver, Timer, Profile>;

    fn enable_motion_control(
        self,
        (timer, profile): (Timer, Profile),
    ) -> Self::WithMotionControl {
        SoftwareMotionControl::new(self, timer, profile)
    }
}
