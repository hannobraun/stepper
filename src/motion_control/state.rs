use core::{
    convert::{TryFrom, TryInto},
    ops,
    task::Poll,
};

use embedded_hal::timer;
use embedded_time::duration::Nanoseconds;
use ramp_maker::MotionProfile;

use crate::{
    traits::{SetDirection, Step},
    Direction, SetDirectionFuture, StepFuture,
};

use super::error::{Error, TimeConversionError};

pub enum State<Driver, Timer, Profile: MotionProfile> {
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

pub fn update<Driver, Timer, Profile>(
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
    Timer::Time: TryFrom<Nanoseconds> + ops::Sub<Output = Timer::Time>,
    Profile: MotionProfile,
    Profile::Delay: TryInto<Timer::Time>,
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
    Time: TryFrom<Nanoseconds> + ops::Sub<Output = Time>,
    Delay: TryInto<Time>,
{
    let delay: Time = delay
        .try_into()
        .map_err(|err| TimeConversionError::DelayToTicks(err))?;
    let pulse_length: Time = pulse_length
        .try_into()
        .map_err(|err| TimeConversionError::NanosecondsToTicks(err))?;

    let delay_left: Time = delay - pulse_length;
    Ok(delay_left)
}
