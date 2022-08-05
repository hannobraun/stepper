use core::task::Poll;

use embedded_hal::digital::blocking::OutputPin;
use embedded_hal::digital::ErrorType;
use fugit::TimerDurationU32 as TimerDuration;
use fugit_timer::Timer as TimerTrait;

use crate::{traits::SetDirection, Direction};

use super::SignalError;

/// The "future" returned by [`Stepper::set_direction`]
///
/// Please note that this type provides a custom API and does not implement
/// [`core::future::Future`]. This might change, when using futures for embedded
/// development becomes more practical.
///
/// [`Stepper::set_direction`]: crate::Stepper::set_direction
#[must_use]
pub struct SetDirectionFuture<Driver, Timer, const TIMER_HZ: u32> {
    direction: Direction,
    driver: Driver,
    timer: Timer,
    state: State,
}

impl<Driver, Timer, const TIMER_HZ: u32>
    SetDirectionFuture<Driver, Timer, TIMER_HZ>
where
    Driver: SetDirection,
    Timer: TimerTrait<TIMER_HZ>,
{
    /// Create new instance of `SetDirectionFuture`
    ///
    /// This constructor is public to provide maximum flexibility for
    /// non-standard use cases. Most users can ignore this and just use
    /// [`Stepper::set_direction`] instead.
    ///
    /// [`Stepper::set_direction`]: crate::Stepper::set_direction
    pub fn new(direction: Direction, driver: Driver, timer: Timer) -> Self {
        Self {
            direction,
            driver,
            timer,
            state: State::Initial,
        }
    }

    /// Poll the future
    ///
    /// The future must be polled for the operation to make progress. The
    /// operation won't start, until this method has been called once. Returns
    /// [`Poll::Pending`], if the operation is not finished yet, or
    /// [`Poll::Ready`], once it is.
    ///
    /// If this method returns [`Poll::Pending`], the user can opt to keep
    /// calling it at a high frequency (see [`Self::wait`]) until the operation
    /// completes, or set up an interrupt that fires once the timer finishes
    /// counting down, and call this method again once it does.
    pub fn poll(
        &mut self,
    ) -> Poll<
        Result<
            (),
            SignalError<
                Driver::Error,
                <Driver::Dir as ErrorType>::Error,
                Timer::Error,
            >,
        >,
    > {
        match self.state {
            State::Initial => {
                match self.direction {
                    Direction::Forward => self
                        .driver
                        .dir()
                        .map_err(|err| SignalError::PinUnavailable(err))?
                        .set_high()
                        .map_err(|err| SignalError::Pin(err))?,
                    Direction::Backward => self
                        .driver
                        .dir()
                        .map_err(|err| SignalError::PinUnavailable(err))?
                        .set_low()
                        .map_err(|err| SignalError::Pin(err))?,
                }

                let ticks: TimerDuration<TIMER_HZ> =
                    Driver::SETUP_TIME.convert();
                self.timer
                    .start(ticks)
                    .map_err(|err| SignalError::Timer(err))?;

                self.state = State::DirectionSet;
                Poll::Pending
            }
            State::DirectionSet => match self.timer.wait() {
                Ok(()) => {
                    self.state = State::Finished;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::Other(err)) => {
                    self.state = State::Finished;
                    Poll::Ready(Err(SignalError::Timer(err)))
                }
                Err(nb::Error::WouldBlock) => Poll::Pending,
            },
            State::Finished => Poll::Ready(Ok(())),
        }
    }

    /// Wait until the operation completes
    ///
    /// This method will call [`Self::poll`] in a busy loop until the operation
    /// has finished.
    pub fn wait(
        &mut self,
    ) -> Result<
        (),
        SignalError<
            Driver::Error,
            <Driver::Dir as ErrorType>::Error,
            Timer::Error,
        >,
    > {
        loop {
            if let Poll::Ready(result) = self.poll() {
                return result;
            }
        }
    }

    /// Drop the future and release the resources that were moved into it
    pub fn release(self) -> (Driver, Timer) {
        (self.driver, self.timer)
    }
}

enum State {
    Initial,
    DirectionSet,
    Finished,
}
