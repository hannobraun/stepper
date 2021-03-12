//! Generic wrapper around a mutable reference
//!
//! See [`RefMut`] for more information.

use embedded_hal::timer;
use embedded_time::duration::Nanoseconds;

use crate::traits::{MotionControl, SetDirection, SetStepMode, Step};

/// Generic wrapper around a mutable reference
///
/// This is used as a means of implementing traits that are already implemented
/// for `T` for `&mut T` too. While this is redundant for the traits from this
/// crate, we couldn't do this for `embedded_hal::timer::CountDown` without a
/// crate-local type.
///
/// The purpose of this is to make the future types more flexible, making it
/// possible to move types into them, or just provide mutable references.
pub struct RefMut<'r, T>(pub &'r mut T);

impl<'r, T> timer::CountDown for RefMut<'r, T>
where
    T: timer::CountDown,
{
    type Error = T::Error;
    type Time = T::Time;

    fn try_start<Time>(&mut self, count: Time) -> Result<(), Self::Error>
    where
        Time: Into<Self::Time>,
    {
        self.0.try_start(count)
    }

    fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
        self.0.try_wait()
    }
}

impl<'r, T> MotionControl for RefMut<'r, T>
where
    T: MotionControl,
{
    type Velocity = T::Velocity;
    type Error = T::Error;

    fn move_to_position(
        &mut self,
        max_velocity: Self::Velocity,
        target_step: i32,
    ) -> Result<(), Self::Error> {
        self.0.move_to_position(max_velocity, target_step)
    }

    fn reset_position(&mut self, step: i32) -> Result<(), Self::Error> {
        self.0.reset_position(step)
    }

    fn update(&mut self) -> Result<bool, Self::Error> {
        self.0.update()
    }
}

impl<'r, T> SetDirection for RefMut<'r, T>
where
    T: SetDirection,
{
    const SETUP_TIME: Nanoseconds = T::SETUP_TIME;

    type Dir = T::Dir;
    type Error = T::Error;

    fn dir(&mut self) -> Result<&mut Self::Dir, Self::Error> {
        self.0.dir()
    }
}

impl<'r, T> SetStepMode for RefMut<'r, T>
where
    T: SetStepMode,
{
    const SETUP_TIME: Nanoseconds = T::SETUP_TIME;
    const HOLD_TIME: Nanoseconds = T::HOLD_TIME;

    type Error = T::Error;
    type StepMode = T::StepMode;

    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error> {
        self.0.apply_mode_config(step_mode)
    }

    fn enable_driver(&mut self) -> Result<(), Self::Error> {
        self.0.enable_driver()
    }
}

impl<'r, T> Step for RefMut<'r, T>
where
    T: Step,
{
    const PULSE_LENGTH: Nanoseconds = T::PULSE_LENGTH;

    type Step = T::Step;
    type Error = T::Error;

    fn step(&mut self) -> Result<&mut Self::Step, Self::Error> {
        self.0.step()
    }
}
