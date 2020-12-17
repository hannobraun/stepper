use core::{
    convert::{TryFrom, TryInto as _},
    task::Poll,
};

use embedded_hal::{digital::OutputPin as _, timer};
use embedded_time::duration::Nanoseconds;
use nb::block;

use crate::{
    traits::{
        EnableDirectionControl, EnableStepControl, EnableStepModeControl,
        SetDirection, SetStepMode, Step,
    },
    Direction,
};

/// Abstract interface to stepper motor drivers
///
/// Wraps a concrete driver and uses the traits that the concrete driver
/// implements to provide an abstract API.
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
pub struct Driver<T> {
    inner: T,
}

impl<T> Driver<T> {
    /// Create a new `Driver` instance from a concrete driver
    pub fn from_inner(inner: T) -> Self {
        Self { inner }
    }

    /// Access a reference to the wrapped driver
    ///
    /// Can be used to access driver-specific functionality that can't be
    /// provided by `Driver`'s abstract interface.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Access a mutable reference to the wrapped driver
    ///
    /// Can be used to access driver-specific functionality that can't be
    /// provided by `Driver`'s abstract interface.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Release the wrapped driver
    ///
    /// Drops this instance of `Driver` and returns the wrapped driver.
    pub fn release(self) -> T {
        self.inner
    }

    /// Enable microstepping mode control
    ///
    /// Consumes `Driver` and returns a new instance that provides control over
    /// the microstepping mode. Once this method has been called, the
    /// [`Driver::set_step_mode`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// microstepping mode as an argument. What exactly those are depends on the
    /// specific driver. Typically they are the output pins that are connected
    /// to the mode pins of the driver.
    ///
    /// This method is only available, if the driver supports enabling step mode
    /// control. It might no longer be available, once step mode control has
    /// been enabled.
    pub fn enable_step_mode_control<Resources, Timer>(
        self,
        res: Resources,
        initial: <T::WithStepModeControl as SetStepMode>::StepMode,
        timer: &mut Timer,
    ) -> Result<
        Driver<T::WithStepModeControl>,
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
        let mut self_ = Driver {
            inner: self.inner.enable_step_mode_control(res),
        };
        self_.set_step_mode(initial, timer)?;

        Ok(self_)
    }

    /// Sets the microstepping mode
    ///
    /// This method is only available, if the wrapped driver supports
    /// microstepping, and supports setting the step mode through software. Some
    /// driver might not support microstepping at all, or only allow setting the
    /// step mode by changing physical switches.
    ///
    /// You might need to call [`Driver::enable_step_mode_control`] to make this
    /// method available.
    pub fn set_step_mode<Timer>(
        &mut self,
        step_mode: T::StepMode,
        timer: &mut Timer,
    ) -> Result<
        (),
        Error<
            T::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    >
    where
        T: SetStepMode,
        Timer: timer::CountDown,
        Timer::Time: TryFrom<Nanoseconds>,
    {
        self.inner
            .apply_mode_config(step_mode)
            .map_err(|err| Error::Pin(err))?;

        let ticks: Timer::Time = T::SETUP_TIME
            .try_into()
            .map_err(|err| Error::TimeConversion(err))?;
        timer.try_start(ticks).map_err(|err| Error::Timer(err))?;
        block!(timer.try_wait()).map_err(|err| Error::Timer(err))?;

        self.inner.enable_driver().map_err(|err| Error::Pin(err))?;

        let ticks: Timer::Time = T::HOLD_TIME
            .try_into()
            .map_err(|err| Error::TimeConversion(err))?;
        timer.try_start(ticks).map_err(|err| Error::Timer(err))?;
        block!(timer.try_wait()).map_err(|err| Error::Timer(err))?;

        Ok(())
    }

    /// Enable direction control
    ///
    /// Consumes `Driver` and returns a new instance that provides control over
    /// the motor direction. Once this method has been called, the
    /// [`Driver::set_direction`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// direction as an argument. What exactly those are depends on the specific
    /// driver. Typically it's going to be the output pin that is connected to
    /// the driver's DIR pin.
    ///
    /// This method is only available, if the driver supports enabling direction
    /// control. It might no longer be available, once direction control has
    /// been enabled.
    pub fn enable_direction_control<Resources, Timer>(
        self,
        res: Resources,
        initial: Direction,
        timer: &mut Timer,
    ) -> Result<
        Driver<T::WithDirectionControl>,
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
        let mut self_ = Driver {
            inner: self.inner.enable_direction_control(res),
        };
        self_.set_direction(initial, timer)?;

        Ok(self_)
    }

    /// Set direction for future movements
    ///
    /// You might need to call [`Driver::enable_direction_control`] to make this
    /// method available.
    pub fn set_direction<Timer>(
        &mut self,
        direction: Direction,
        timer: &mut Timer,
    ) -> Result<
        (),
        Error<
            T::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    >
    where
        T: SetDirection,
        Timer: timer::CountDown,
        Timer::Time: TryFrom<Nanoseconds>,
    {
        match direction {
            Direction::Forward => self
                .inner
                .dir()
                .try_set_high()
                .map_err(|err| Error::Pin(err))?,
            Direction::Backward => self
                .inner
                .dir()
                .try_set_low()
                .map_err(|err| Error::Pin(err))?,
        }

        let ticks: Timer::Time = T::SETUP_TIME
            .try_into()
            .map_err(|err| Error::TimeConversion(err))?;
        timer.try_start(ticks).map_err(|err| Error::Timer(err))?;
        block!(timer.try_wait()).map_err(|err| Error::Timer(err))?;

        Ok(())
    }

    /// Enable step control
    ///
    /// Consumes `Driver` and returns a new instance that provides control over
    /// stepping the motor. Once this method has been called, the
    /// [`Driver::step`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// direction as an argument. What exactly those are depends on the specific
    /// driver. Typically it's going to be the output pin that is connected to
    /// the driver's STEP pin.
    ///
    /// This method is only available, if the driver supports enabling step
    /// control. It might no longer be available, once step control has been
    /// enabled.
    pub fn enable_step_control<Resources>(
        self,
        res: Resources,
    ) -> Driver<T::WithStepControl>
    where
        T: EnableStepControl<Resources>,
    {
        Driver {
            inner: self.inner.enable_step_control(res),
        }
    }

    /// Rotates the motor one (micro-)step in the given direction
    ///
    /// Steps the motor one step in the direction that was previously set,
    /// according to current microstep configuration. To achieve a specific
    /// speed, the user must call this method at the appropriate frequency.
    ///
    /// You might need to call [`Driver::enable_step_control`] to make this
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
        StepFuture {
            driver: self,
            timer,
            state: StepState::Initial,
        }
    }

    /// Returns the step pulse length of the wrapped driver
    pub fn pulse_length(&self) -> Nanoseconds
    where
        T: Step,
    {
        T::PULSE_LENGTH
    }
}

/// A "future" that can be polled to complete a [`Driver::step`] call
///
/// Please note that this type provides a custom API and does not implement
/// [`core::future::Future`]. This might change, as using futures for embedded
/// development becomes more practical.
pub struct StepFuture<'r, T, Timer> {
    driver: &'r mut Driver<T>,
    timer: &'r mut Timer,
    state: StepState,
}

impl<T, Timer> StepFuture<'_, T, Timer>
where
    T: Step,
    Timer: timer::CountDown,
    Timer::Time: TryFrom<Nanoseconds>,
{
    /// Poll the future
    ///
    /// The future must be polled for the operation to make progress. It must be
    /// called once, to start the operation.
    ///
    /// Once the operation has been started and this method returns
    /// [`Poll::Pending`], the user can opt to keep calling it at a high
    /// frequency (for example in a busy loop) until the future finishes.
    /// Alternatively, the user can set up an interrupt that fires once the
    /// timer finishes counting down, and call this method again once it does.
    ///
    /// This will return `Poll::Pending`, if the operation has not completed
    /// yet, or `Poll::Ready`, once it has.
    pub fn poll(
        &mut self,
    ) -> Poll<
        Result<
            (),
            Error<
                T::Error,
                <Timer::Time as TryFrom<Nanoseconds>>::Error,
                Timer::Error,
            >,
        >,
    > {
        match self.state {
            StepState::Initial => {
                // Start step pulse
                self.driver
                    .inner
                    .step()
                    .try_set_high()
                    .map_err(|err| Error::Pin(err))?;

                let ticks: Timer::Time = T::PULSE_LENGTH
                    .try_into()
                    .map_err(|err| Error::TimeConversion(err))?;
                self.timer
                    .try_start(ticks)
                    .map_err(|err| Error::Timer(err))?;

                self.state = StepState::PulseStarted;
                Poll::Pending
            }
            StepState::PulseStarted => {
                match self.timer.try_wait() {
                    Ok(()) => {
                        // End step pulse
                        self.driver
                            .inner
                            .step()
                            .try_set_low()
                            .map_err(|err| Error::Pin(err))?;

                        self.state = StepState::Finished;
                        Poll::Ready(Ok(()))
                    }
                    Err(nb::Error::Other(err)) => {
                        self.state = StepState::Finished;
                        Poll::Ready(Err(Error::Timer(err)))
                    }
                    Err(nb::Error::WouldBlock) => Poll::Pending,
                }
            }
            StepState::Finished => Poll::Ready(Ok(())),
        }
    }

    /// Wait until the operation has finished
    ///
    /// This method will call [`StepFuture::poll`] in a busy loop until the
    /// operation has finished.
    pub fn wait(
        &mut self,
    ) -> Result<
        (),
        Error<
            T::Error,
            <Timer::Time as TryFrom<Nanoseconds>>::Error,
            Timer::Error,
        >,
    > {
        loop {
            if let Poll::Ready(result) = self.poll() {
                return result;
            }
        }
    }
}

enum StepState {
    Initial,
    PulseStarted,
    Finished,
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
