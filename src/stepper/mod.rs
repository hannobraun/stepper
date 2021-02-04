mod set_direction;
mod set_step_mode;
mod step;

pub use self::{
    set_direction::SetDirectionFuture, set_step_mode::SetStepModeFuture,
    step::StepFuture,
};

use core::convert::TryFrom;

use embedded_hal::timer;
use embedded_time::duration::Nanoseconds;

use crate::{
    traits::{
        EnableDirectionControl, EnableStepControl, EnableStepModeControl,
        SetDirection, SetStepMode, Step,
    },
    Direction,
};

/// Abstract interface to stepper motors
///
/// Wraps a concrete stepper driver or controller, and uses the traits that this
/// concrete driver or controller implements to provide an abstract API. You can
/// construct an instance of this type using [`Stepper::from_inner`].
///
/// # Notes on timer use
///
/// Some of this struct's methods take a timer argument. This is expected to be
/// an implementation of [`embedded_hal::timer::CountDown`], with the additional
/// requirement that `CountDown::Time` has a `TryFrom<Nanoseconds>`
/// implementation, where `Nanoseconds` refers to
/// [`embedded_time::duration::Nanoseconds`].
///
/// Not every `CountDown` implementation provides this for its `Time` type, so
/// it might be necessary that the user either adds this `embedded_time`
/// integration to the HAL library they are using, or provides a wrapper around
/// the `CountDown` implementation in their own code, adding the conversion
/// there.
///
/// Every method that takes a timer argument internally performs the conversion
/// from `Nanoseconds` to the timers `Time` type. Since the nanosecond values
/// are constant and the `CountDown` implementation is known statically, the
/// compiler should have enough information to perform this conversion at
/// compile-time.
///
/// Unfortunately there is currently no way to make sure that this optimization
/// actually happens. Additions like [RFC 2632], [RFC 2920], and possibly others
/// along those lines, could help with this in the future. For now, users must
/// manually inspect the generated code and tweak optimization settings (and
/// possibly the HAL-specific conversion code), if this level of performance is
/// required.
///
/// [RFC 2632]: https://github.com/rust-lang/rfcs/pull/2632
/// [RFC 2920]: https://github.com/rust-lang/rfcs/pull/2920
pub struct Stepper<T> {
    inner: T,
}

impl<T> Stepper<T> {
    /// Create a new `Stepper` instance from a concrete driver or controller
    pub fn from_inner(inner: T) -> Self {
        Self { inner }
    }

    /// Access a reference to the wrapped driver or controller
    ///
    /// Can be used to access driver/controller-specific functionality that
    /// can't be provided by `Stepper`'s abstract interface.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Access a mutable reference to the wrapped driver or controller
    ///
    /// Can be used to access driver/controller-specific functionality that
    /// can't be provided by `Stepper`'s abstract interface.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Release the wrapped driver or controller
    ///
    /// Drops this instance of `Stepper` and returns the wrapped driver/
    /// controller.
    pub fn release(self) -> T {
        self.inner
    }

    /// Enable microstepping mode control
    ///
    /// Consumes this instance of `Stepper` and returns a new instance that
    /// provides control over the microstepping mode. Once this method has been
    /// called, the [`Stepper::set_step_mode`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// microstepping mode as an argument. What exactly those are depends on the
    /// specific driver/controller. Typically they are the output pins that are
    /// connected to the mode pins of the driver/controller.
    ///
    /// This method is only available, if the driver/controller supports
    /// enabling step mode control. It might no longer be available, once step
    /// mode control has been enabled.
    pub fn enable_step_mode_control<Resources, Timer>(
        self,
        res: Resources,
        initial: <T::WithStepModeControl as SetStepMode>::StepMode,
        timer: &mut Timer,
    ) -> Result<
        Stepper<T::WithStepModeControl>,
        Error<
            <T::WithStepModeControl as SetStepMode>::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    >
    where
        T: EnableStepModeControl<Resources>,
        Timer: timer::CountDown,
        Timer::Time: TryFrom<Nanoseconds>,
    {
        let mut self_ = Stepper {
            inner: self.inner.enable_step_mode_control(res),
        };
        self_.set_step_mode(initial, timer).wait()?;

        Ok(self_)
    }

    /// Sets the microstepping mode
    ///
    /// This method is only available, if the wrapped driver/controller supports
    /// microstepping, and supports setting the step mode through software. Some
    /// drivers/controllers might not support microstepping at all, or only
    /// allow setting the step mode by changing physical switches.
    ///
    /// You might need to call [`Stepper::enable_step_mode_control`] to make
    /// this method available.
    pub fn set_step_mode<'r, Timer>(
        &'r mut self,
        step_mode: T::StepMode,
        timer: &'r mut Timer,
    ) -> SetStepModeFuture<'r, T, Timer>
    where
        T: SetStepMode,
        Timer: timer::CountDown,
        Timer::Time: TryFrom<Nanoseconds>,
    {
        SetStepModeFuture::new(step_mode, self, timer)
    }

    /// Enable direction control
    ///
    /// Consumes this instance of `Stepper` and returns a new instance that
    /// provides control over the motor direction. Once this method has been
    /// called, the [`Stepper::set_direction`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// direction as an argument. What exactly those are depends on the specific
    /// driver/controller. Typically it's going to be the output pin that is
    /// connected to the driver/controller's DIR pin.
    ///
    /// This method is only available, if the driver/controller supports
    /// enabling direction control. It might no longer be available, once
    /// direction control has been enabled.
    pub fn enable_direction_control<Resources, Timer>(
        self,
        res: Resources,
        initial: Direction,
        timer: &mut Timer,
    ) -> Result<
        Stepper<T::WithDirectionControl>,
        Error<
            <T::WithDirectionControl as SetDirection>::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    >
    where
        T: EnableDirectionControl<Resources>,
        Timer: timer::CountDown,
        Timer::Time: TryFrom<Nanoseconds>,
    {
        let mut self_ = Stepper {
            inner: self.inner.enable_direction_control(res),
        };
        self_.set_direction(initial, timer).wait()?;

        Ok(self_)
    }

    /// Set direction for future movements
    ///
    /// You might need to call [`Stepper::enable_direction_control`] to make
    /// this method available.
    pub fn set_direction<'r, Timer>(
        &'r mut self,
        direction: Direction,
        timer: &'r mut Timer,
    ) -> SetDirectionFuture<'r, T, Timer>
    where
        T: SetDirection,
        Timer: timer::CountDown,
        Timer::Time: TryFrom<Nanoseconds>,
    {
        SetDirectionFuture::new(direction, self, timer)
    }

    /// Enable step control
    ///
    /// Consumes this instance of `Stepper` and returns a new instance that
    /// provides control over stepping the motor. Once this method has been
    /// called, the [`Stepper::step`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// direction as an argument. What exactly those are depends on the specific
    /// driver/controller. Typically it's going to be the output pin that is
    /// connected to the driver/controller's STEP pin.
    ///
    /// This method is only available, if the driver/controller supports
    /// enabling step control. It might no longer be available, once step
    /// control has been enabled.
    pub fn enable_step_control<Resources>(
        self,
        res: Resources,
    ) -> Stepper<T::WithStepControl>
    where
        T: EnableStepControl<Resources>,
    {
        Stepper {
            inner: self.inner.enable_step_control(res),
        }
    }

    /// Rotates the motor one (micro-)step in the given direction
    ///
    /// Steps the motor one step in the direction that was previously set,
    /// according to current microstepping configuration. To achieve a specific
    /// speed, the user must call this method at an appropriate frequency.
    ///
    /// You might need to call [`Stepper::enable_step_control`] to make this
    /// method available.
    pub fn step<'r, Timer>(
        &'r mut self,
        timer: &'r mut Timer,
    ) -> StepFuture<'r, T, Timer>
    where
        T: Step,
        Timer: timer::CountDown,
        Timer::Time: TryFrom<Nanoseconds>,
    {
        StepFuture::new(self, timer)
    }

    /// Returns the step pulse length of the wrapped driver/controller
    ///
    /// The pulse length is also available through the [`Step`] trait. This
    /// method provides a more convenient way to access it.
    pub fn pulse_length(&self) -> Nanoseconds
    where
        T: Step,
    {
        T::PULSE_LENGTH
    }
}

/// An error that can occur while using this API
#[derive(Debug, Eq, PartialEq)]
pub enum Error<PinError, TimeConversionError, TimerError> {
    /// An error originated from using the [`OutputPin`] trait
    ///
    /// [`OutputPin`]: embedded_hal::digital::OutputPin
    Pin(PinError),

    /// An error occurred while converting time to timer ticks
    TimeConversion(TimeConversionError),

    /// An error originated from working with a timer
    Timer(TimerError),
}
