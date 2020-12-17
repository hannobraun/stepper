use core::{
    convert::{TryFrom, TryInto as _},
    task::Poll,
};

use embedded_hal::{prelude::*, timer};
use embedded_time::duration::Nanoseconds;

use crate::{traits::SetDirection, Direction};

use super::{Driver, Error};

/// A "future" that can be polled to complete a [`Driver::set_direction`] call
///
/// Please note that this type provides a custom API and does not implement
/// [`core::future::Future`]. This might change, as using futures for embedded
/// development becomes more practical.
pub struct SetDirectionFuture<'r, T, Timer> {
    direction: Direction,
    driver: &'r mut Driver<T>,
    timer: &'r mut Timer,
    state: State,
}

impl<'r, T, Timer> SetDirectionFuture<'r, T, Timer>
where
    T: SetDirection,
    Timer: timer::CountDown,
    Timer::Time: TryFrom<Nanoseconds>,
{
    pub(super) fn new(
        direction: Direction,
        driver: &'r mut Driver<T>,
        timer: &'r mut Timer,
    ) -> Self {
        Self {
            direction,
            driver,
            timer,
            state: State::Initial,
        }
    }

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
            State::Initial => {
                match self.direction {
                    Direction::Forward => self
                        .driver
                        .inner
                        .dir()
                        .try_set_high()
                        .map_err(|err| Error::Pin(err))?,
                    Direction::Backward => self
                        .driver
                        .inner
                        .dir()
                        .try_set_low()
                        .map_err(|err| Error::Pin(err))?,
                }

                let ticks: Timer::Time = T::SETUP_TIME
                    .try_into()
                    .map_err(|err| Error::TimeConversion(err))?;
                self.timer
                    .try_start(ticks)
                    .map_err(|err| Error::Timer(err))?;

                self.state = State::DirectionSet;
                Poll::Pending
            }
            State::DirectionSet => match self.timer.try_wait() {
                Ok(()) => {
                    self.state = State::Finished;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::Other(err)) => {
                    self.state = State::Finished;
                    Poll::Ready(Err(Error::Timer(err)))
                }
                Err(nb::Error::WouldBlock) => Poll::Pending,
            },
            State::Finished => Poll::Ready(Ok(())),
        }
    }

    /// Wait until the operation has finished
    ///
    /// This method will call [`SetDirectionFuture::poll`] in a busy loop until
    /// the operation has finished.
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

enum State {
    Initial,
    DirectionSet,
    Finished,
}
