mod error;
mod move_to;
mod set_direction;
mod set_step_mode;
mod step;

pub use self::{
    error::{Error, SignalError},
    move_to::MoveToFuture,
    set_direction::SetDirectionFuture,
    set_step_mode::SetStepModeFuture,
    step::StepFuture,
};

use core::convert::Infallible;

use embedded_hal::digital::blocking::OutputPin;
use fugit::NanosDurationU32 as Nanoseconds;
use fugit_timer::Timer as TimerTrait;

use crate::{
    traits::{
        EnableDirectionControl, EnableMotionControl, EnableStepControl,
        EnableStepModeControl, MotionControl, SetDirection, SetStepMode, Step,
    },
    util::ref_mut::RefMut,
    Direction,
};

/// Unified stepper motor interface
///
/// Wraps a driver that interfaces with the motor-controlling hardware and
/// abstracts over it, providing an interface that works the same, no matter
/// what kind of hardware controls the stepper motor.
///
/// You can construct an instance of this type using [`Stepper::from_driver`].
///
/// # Nomenclature
///
/// This structs wraps a software component that interfaces with hardware that
/// controls a stepper motor. That software component is called a "driver",
/// because it "drives" the hardware it interfaces with.
///
/// The driven hardware typically comes in two forms:
///
/// - A low-level chip controlled by STEP and DIR signals, often called a
///   stepper driver (yes, somewhat confusing) or stepper controller.
/// - A higher-level chip, typically controlled through some serial interface,
///   often called a motion controller.
///
/// In practice, a given product can cleanly fall into one of the two camps,
/// both, or anything in between.
///
/// # Hardware capabilities
///
/// Depending on the actual hardware we're interfacing with, we might only have
/// access to the bare minimum functionality (STEP and DIR pins) or high-level
/// motion control features. Since `Stepper` is agnostic on the driver and the
/// hardware it interfaces with, there must be a way to deal with those
/// differing capabilities.
///
/// `Stepper` provides a number of `enable_*` methods that enable access to a
/// specific hardware capability, if the hardware and driver support this. Once
/// that method has been called, the methods that control the hardware
/// capability are available.
///
/// ## Step mode control
///
/// Enable this capability with [`Stepper::enable_step_mode_control`] and use it
/// with [`Stepper::set_step_mode`]. Since not all stepper drivers support
/// microstepping and of those that do, not all support setting it from
/// software, this capability might not be available for all drivers.
///
/// ## Direction control & step control
///
/// Enable direction control with [`Stepper::enable_direction_control`] and use
/// it with [`Stepper::set_direction`]. Enable step control with
/// [`Stepper::enable_step_control`] and use ith with [`Stepper::step`].
///
/// These capabilities are supported by virtually all stepper drivers, but might
/// not be available for motion controllers. Where they are available, they are
/// typically available together. They are modeled as separate capabilities, as
/// to not make any assumptions. If you want to generate steps from software,
/// for example, but control direction via some other means, then you can.
///
/// ## Motion control
///
/// Enable motion control with [`Stepper::enable_motion_control`] and use it
/// with [`Stepper::move_to_position`] and [`Stepper::reset_position`].
///
/// Motion control capability is directly supported by motion control chips, but
/// a software implementation based on direction and step control exists in the
/// [`motion_control`] module, to make the capability available for all drivers.
///
/// [`motion_control`]: crate::motion_control
///
/// # Notes on timer use
///
/// Some of this struct's methods take a timer argument. This is expected to be
/// an implementation of [`fugit_timer::Timer`].
///
pub struct Stepper<Driver> {
    driver: Driver,
}

impl<Driver> Stepper<Driver> {
    /// Create a new `Stepper` instance from a driver
    pub fn from_driver(driver: Driver) -> Self {
        Self { driver }
    }

    /// Access a reference to the wrapped driver
    ///
    /// Can be used to access driver-specific functionality that can't be
    /// provided by `Stepper`'s abstract interface.
    pub fn driver(&self) -> &Driver {
        &self.driver
    }

    /// Access a mutable reference to the wrapped driver
    ///
    /// Can be used to access driver-specific functionality that can't be
    /// provided by `Stepper`'s abstract interface.
    pub fn driver_mut(&mut self) -> &mut Driver {
        &mut self.driver
    }

    /// Release the wrapped driver
    ///
    /// Drops this instance of `Stepper` and returns the wrapped driver.
    pub fn release(self) -> Driver {
        self.driver
    }

    /// Enable microstepping mode control
    ///
    /// Consumes this instance of `Stepper` and returns a new instance that
    /// provides control over the microstepping mode. Once this method has been
    /// called, the [`Stepper::set_step_mode`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// microstepping mode as an argument. What exactly those are depends on the
    /// specific driver. Typically they are the output pins that are connected
    /// to the mode pins of the driver.
    ///
    /// This method is only available, if the driver supports enabling step mode
    /// control. It might no longer be available, once step mode control has
    /// been enabled.
    pub fn enable_step_mode_control<Resources, Timer, const TIMER_HZ: u32>(
        self,
        res: Resources,
        initial: <Driver::WithStepModeControl as SetStepMode>::StepMode,
        timer: &mut Timer,
    ) -> Result<
        Stepper<Driver::WithStepModeControl>,
        SignalError<
            Infallible, // only applies to `SetDirection`, `Step`
            <Driver::WithStepModeControl as SetStepMode>::Error,
            Timer::Error,
        >,
    >
    where
        Driver: EnableStepModeControl<Resources>,
        Timer: TimerTrait<TIMER_HZ>,
    {
        let mut self_ = Stepper {
            driver: self.driver.enable_step_mode_control(res),
        };
        self_.set_step_mode(initial, timer).wait()?;

        Ok(self_)
    }

    /// Sets the microstepping mode
    ///
    /// This method is only available, if the wrapped driver supports
    /// microstepping, and supports setting the step mode through software. Some
    /// hardware might not support microstepping at all, or only allow setting
    /// the step mode by changing physical switches.
    ///
    /// You might need to call [`Stepper::enable_step_mode_control`] to make
    /// this method available.
    pub fn set_step_mode<'r, Timer, const TIMER_HZ: u32>(
        &'r mut self,
        step_mode: Driver::StepMode,
        timer: &'r mut Timer,
    ) -> SetStepModeFuture<RefMut<'r, Driver>, RefMut<'r, Timer>, TIMER_HZ>
    where
        Driver: SetStepMode,
        Timer: TimerTrait<TIMER_HZ>,
    {
        SetStepModeFuture::new(
            step_mode,
            RefMut(&mut self.driver),
            RefMut(timer),
        )
    }

    /// Enable direction control
    ///
    /// Consumes this instance of `Stepper` and returns a new instance that
    /// provides control over the motor direction. Once this method has been
    /// called, the [`Stepper::set_direction`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// direction as an argument. What exactly those are depends on the specific
    /// driver. Typically it's going to be the output pin that is connected to
    /// the hardware's DIR pin.
    ///
    /// This method is only available, if the driver supports enabling direction
    /// control. It might no longer be available, once direction control has
    /// been enabled.
    pub fn enable_direction_control<Resources, Timer, const TIMER_HZ: u32>(
        self,
        res: Resources,
        initial: Direction,
        timer: &mut Timer,
    ) -> Result<
        Stepper<Driver::WithDirectionControl>,
        SignalError<
            <Driver::WithDirectionControl as SetDirection>::Error,
            <<Driver::WithDirectionControl as SetDirection>::Dir
                as OutputPin>::Error,
            Timer::Error,
        >,
    >
    where
        Driver: EnableDirectionControl<Resources>,
        Timer: TimerTrait<TIMER_HZ>,
    {
        let mut self_ = Stepper {
            driver: self.driver.enable_direction_control(res),
        };
        self_.set_direction(initial, timer).wait()?;

        Ok(self_)
    }

    /// Set direction for future movements
    ///
    /// You might need to call [`Stepper::enable_direction_control`] to make
    /// this method available.
    pub fn set_direction<'r, Timer, const TIMER_HZ: u32>(
        &'r mut self,
        direction: Direction,
        timer: &'r mut Timer,
    ) -> SetDirectionFuture<RefMut<'r, Driver>, RefMut<'r, Timer>, TIMER_HZ>
    where
        Driver: SetDirection,
        Timer: TimerTrait<TIMER_HZ>,
    {
        SetDirectionFuture::new(
            direction,
            RefMut(&mut self.driver),
            RefMut(timer),
        )
    }

    /// Enable step control
    ///
    /// Consumes this instance of `Stepper` and returns a new instance that
    /// provides control over stepping the motor. Once this method has been
    /// called, the [`Stepper::step`] method becomes available.
    ///
    /// Takes the hardware resources that are required for controlling the
    /// stepping as an argument. What exactly those are depends on the specific
    /// driver. Typically it's going to be the output pin that is connected to
    /// the hardware's STEP pin.
    ///
    /// This method is only available, if the driver/controller supports
    /// enabling step control. It might no longer be available, once step
    /// control has been enabled.
    pub fn enable_step_control<Resources>(
        self,
        res: Resources,
    ) -> Stepper<Driver::WithStepControl>
    where
        Driver: EnableStepControl<Resources>,
    {
        Stepper {
            driver: self.driver.enable_step_control(res),
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
    pub fn step<'r, Timer, const TIMER_HZ: u32>(
        &'r mut self,
        timer: &'r mut Timer,
    ) -> StepFuture<RefMut<'r, Driver>, RefMut<'r, Timer>, TIMER_HZ>
    where
        Driver: Step,
        Timer: TimerTrait<TIMER_HZ>,
    {
        StepFuture::new(RefMut(&mut self.driver), RefMut(timer))
    }

    /// Returns the step pulse length of the wrapped driver/controller
    ///
    /// The pulse length is also available through the [`Step`] trait. This
    /// method provides a more convenient way to access it.
    ///
    /// You might need to call [`Stepper::enable_step_control`] to make this
    /// method available.
    pub fn pulse_length(&self) -> Nanoseconds
    where
        Driver: Step,
    {
        Driver::PULSE_LENGTH
    }

    /// Enable motion control
    ///
    /// Consumes this instance of `Stepper` and returns a new instance that
    /// provides motion control capabilities. Once this method has been called,
    /// the motion control API ([`Stepper::move_to_position`],
    /// [`Stepper::reset_position`]) becomes available.
    ///
    /// Takes the hardware resources that are required for motion control as an
    /// argument. What exactly those are depends on the specific driver.
    /// Typically it's either going to be some kind of communication interface,
    /// for drivers that have access to hardware support for motion control, or
    /// a motion profile from the RampMaker library, for drivers that have
    /// support for setting direction and stepping and require a software
    /// fallback for motion control.
    ///
    /// This method should be available for virtually all drivers, either via
    /// hardware support, or through the aforementioned software fallback. It
    /// might no longer be available, once motion control support has been
    /// enabled.
    pub fn enable_motion_control<Resources, const TIMER_HZ: u32>(
        self,
        res: Resources,
    ) -> Stepper<Driver::WithMotionControl>
    where
        Driver: EnableMotionControl<Resources, TIMER_HZ>,
    {
        Stepper {
            driver: self.driver.enable_motion_control(res),
        }
    }

    /// Move the motor to the given position
    ///
    /// Moves the motor to the given position (`target_step`), while respecting
    /// the maximum velocity (`max_velocity`). The specifics of the motion
    /// profile (like acceleration and jerk) are driver-defined.
    ///
    /// It might be possible to influence the parameters of the motion profile
    /// through the resources passed to [`Stepper::enable_motion_control`],
    /// which might include configuration.
    ///
    /// To modify on ongoing movement, you can drop the future returned by this
    /// method and call it again with different parameters (or call another
    /// method).
    ///
    /// You might need to call [`Stepper::enable_motion_control`] to make this
    /// method available.
    pub fn move_to_position<'r>(
        &'r mut self,
        max_velocity: Driver::Velocity,
        target_step: i32,
    ) -> MoveToFuture<RefMut<'r, Driver>>
    where
        Driver: MotionControl,
    {
        MoveToFuture::new(RefMut(&mut self.driver), max_velocity, target_step)
    }

    /// Reset the position to the given value
    ///
    /// This should never result in a movement, as this method only overwrites
    /// the internal position counter of the driver. However, it might influence
    /// an already ongoing movement.
    ///
    /// You might need to call [`Stepper::enable_motion_control`] to make this
    /// method available.
    pub fn reset_position(&mut self, step: i32) -> Result<(), Driver::Error>
    where
        Driver: MotionControl,
    {
        self.driver.reset_position(step)
    }
}
