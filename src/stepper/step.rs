use core::{
    convert::{TryFrom, TryInto as _},
    task::Poll,
};

use embedded_hal::{prelude::*, timer};
use embedded_time::duration::Nanoseconds;

use crate::traits::Step;

use super::Error;

/// The "future" returned by [`Stepper::step`]
///
/// Please note that this type provides a custom API and does not implement
/// [`core::future::Future`]. This might change, when using futures for embedded
/// development becomes more practical.
///
/// [`Stepper::step`]: crate::Stepper::step
pub struct StepFuture<'r, Driver, Timer> {
    driver: &'r mut Driver,
    timer: &'r mut Timer,
    state: State,
}

impl<'r, Driver, Timer> StepFuture<'r, Driver, Timer>
where
    Driver: Step,
    Timer: timer::CountDown,
    Timer::Time: TryFrom<Nanoseconds>,
{
    pub(super) fn new(driver: &'r mut Driver, timer: &'r mut Timer) -> Self {
        Self {
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
            Error<
                Driver::Error,
                <Timer::Time as TryFrom<Nanoseconds>>::Error,
                Timer::Error,
            >,
        >,
    > {
        match self.state {
            State::Initial => {
                // Start step pulse
                self.driver
                    .step()
                    .try_set_high()
                    .map_err(|err| Error::Pin(err))?;

                let ticks: Timer::Time = Driver::PULSE_LENGTH
                    .try_into()
                    .map_err(|err| Error::TimeConversion(err))?;
                self.timer
                    .try_start(ticks)
                    .map_err(|err| Error::Timer(err))?;

                self.state = State::PulseStarted;
                Poll::Pending
            }
            State::PulseStarted => {
                match self.timer.try_wait() {
                    Ok(()) => {
                        // End step pulse
                        self.driver
                            .step()
                            .try_set_low()
                            .map_err(|err| Error::Pin(err))?;

                        self.state = State::Finished;
                        Poll::Ready(Ok(()))
                    }
                    Err(nb::Error::Other(err)) => {
                        self.state = State::Finished;
                        Poll::Ready(Err(Error::Timer(err)))
                    }
                    Err(nb::Error::WouldBlock) => Poll::Pending,
                }
            }
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
        Error<
            Driver::Error,
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
    PulseStarted,
    Finished,
}
