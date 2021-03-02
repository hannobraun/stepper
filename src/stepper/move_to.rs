use core::task::Poll;

use crate::traits::MotionControl;

/// The "future" returned by [`Stepper::move_to_position`]
///
/// Please note that this type provides a custom API and does not implement
/// [`core::future::Future`]. This might change, when using futures for embedded
/// development becomes more practical.
///
/// [`Stepper::move_to_position`]: crate::Stepper::move_to_position
#[must_use]
pub struct MoveToFuture<Driver: MotionControl> {
    driver: Driver,
    state: State<Driver::Velocity>,
}

impl<Driver> MoveToFuture<Driver>
where
    Driver: MotionControl,
{
    /// Create new instance of `MoveToFuture`
    ///
    /// This constructor is public to provide maximum flexibility for
    /// non-standard use cases. Most users can ignore this and just use
    /// [`Stepper::move_to_position`] instead.
    ///
    /// [`Stepper::move_to_position`]: crate::Stepper::move_to_position
    pub fn new(
        driver: Driver,
        max_velocity: Driver::Velocity,
        target_step: i32,
    ) -> Self {
        Self {
            driver,
            state: State::Initial {
                max_velocity,
                target_step,
            },
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
    pub fn poll(&mut self) -> Poll<Result<(), Driver::Error>> {
        match self.state {
            State::Initial {
                max_velocity,
                target_step,
            } => {
                self.driver.move_to_position(max_velocity, target_step)?;
                self.state = State::Moving;
                Poll::Pending
            }
            State::Moving => {
                let still_moving = self.driver.update()?;
                if still_moving {
                    Poll::Pending
                } else {
                    self.state = State::Finished;
                    Poll::Ready(Ok(()))
                }
            }
            State::Finished => Poll::Ready(Ok(())),
        }
    }

    /// Wait until the operation completes
    ///
    /// This method will call [`Self::poll`] in a busy loop until the operation
    /// has finished.
    pub fn wait(&mut self) -> Result<(), Driver::Error> {
        loop {
            if let Poll::Ready(result) = self.poll() {
                return result;
            }
        }
    }

    /// Drop the future and release the resources that were moved into it
    pub fn release(self) -> Driver {
        self.driver
    }
}

enum State<Velocity> {
    Initial {
        max_velocity: Velocity,
        target_step: i32,
    },
    Moving,
    Finished,
}
