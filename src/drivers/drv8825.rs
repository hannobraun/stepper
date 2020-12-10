//! DRV8825 Driver
//!
//! Platform-agnostic driver for the DRV8825 stepper motor driver. This module
//! can be used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! The entry point to this module is the [`DRV8825`] struct.
//!
//! # Example
//!
//! ``` rust
//! # fn main()
//! #     -> Result<
//! #         (),
//! #         step_dir::StepError<core::convert::Infallible>
//! #     > {
//! #
//! use step_dir::{
//!     embedded_time::{duration::Microseconds, Clock as _},
//!     drivers::drv8825::DRV8825,
//!     Direction, Driver,
//! };
//!
//! const STEP_DELAY: Microseconds = Microseconds(500);
//!
//! # struct Pin;
//! # impl step_dir::embedded_hal::digital::OutputPin for Pin {
//! #     type Error = core::convert::Infallible;
//! #     fn try_set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn try_set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! #
//! # struct Clock(std::time::Instant);
//! # impl step_dir::embedded_time::Clock for Clock {
//! #     type T = u32;
//! #     const SCALING_FACTOR: step_dir::embedded_time::fraction::Fraction =
//! #         step_dir::embedded_time::fraction::Fraction::new(1, 1_000_000);
//! #     fn try_now(&self)
//! #         -> Result<
//! #             step_dir::embedded_time::Instant<Self>,
//! #             step_dir::embedded_time::clock::Error
//! #         >
//! #     {
//! #         Ok(
//! #             step_dir::embedded_time::Instant::new(
//! #                 self.0.elapsed().as_micros() as u32
//! #             )
//! #         )
//! #     }
//! # }
//! #
//! # let step = Pin;
//! # let dir = Pin;
//! # let mut clock = Clock(std::time::Instant::now());
//! #
//! // You need to acquire the GPIO pins connected to the STEP and DIR signals.
//! // How you do this depends on your target platform. All the driver cares
//! // about is that they implement `embedded_hal::digital::OutputPin`. You also
//! // need an implementation of `embedded_hal::blocking::DelayUs`.
//!
//! // Create driver API from STEP and DIR pins.
//! let mut driver = Driver::new(
//!     DRV8825::from_step_dir_pins(step, dir)
//! );
//!
//! // Rotate stepper motor by a few steps.
//! driver.set_direction(Direction::Forward, &clock)?;
//! for _ in 0 .. 5 {
//!     let timer = clock.new_timer(STEP_DELAY).start()?;
//!     driver.step(&clock)?;
//!     timer.wait()?;
//! }
//!
//! #
//! # Ok(())
//! # }
//! ```
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal

use embedded_hal::digital::{OutputPin, PinState};
use embedded_time::{duration::Nanoseconds, Clock};

use crate::{
    traits::{Dir as DirTrait, SetStepMode, Step as StepTrait},
    ModeError, StepMode32,
};

/// The DRV8825 driver API
///
/// You can create an instance of this struct by calling
/// [`DRV8825::from_step_dir_pins`]. See [module documentation] for a full
/// example that uses this API.
///
/// [module documentation]: index.html
pub struct DRV8825<Enable, Fault, Sleep, Reset, Mode0, Mode1, Mode2, Step, Dir>
{
    enable: Enable,
    fault: Fault,
    sleep: Sleep,
    reset: Reset,
    mode0: Mode0,
    mode1: Mode1,
    mode2: Mode2,
    step: Step,
    dir: Dir,
}

impl<Step, Dir> DRV8825<(), (), (), (), (), (), (), Step, Dir> {
    /// Create a new instance of `DRV8825`
    ///
    /// Creates an instance of this struct from just the STEP and DIR pins. It
    /// expects the types that represent those pins to implement [`OutputPin`].
    ///
    /// The resulting instance can be used to step the motor using
    /// [`DRV8825::step`]. All other capabilities of the DRV8825, like
    /// the power-up sequence, selecting a step mode, or controlling the power
    /// state, explicitly enabled, or managed externally.
    ///
    /// To enable additional capabilities, see
    /// [`DRV8825::enable_mode_control`].
    pub fn from_step_dir_pins<Error>(step: Step, dir: Dir) -> Self
    where
        Step: OutputPin<Error = Error>,
        Dir: OutputPin<Error = Error>,
    {
        Self {
            enable: (),
            fault: (),
            sleep: (),
            reset: (),
            mode0: (),
            mode1: (),
            mode2: (),
            step,
            dir,
        }
    }
}

impl<Step, Dir> DRV8825<(), (), (), (), (), (), (), Step, Dir> {
    /// Enables support for step mode control and sets the initial step mode
    ///
    /// Consumes this instance of `DRV8825` and returns another instance that
    /// has support for controlling the step mode. Requires the additional pins
    /// for doing so, namely RESET, MODE0, MODE1, and MODE2. It expects the
    /// types that represent those pins to implement [`OutputPin`].
    ///
    /// This method is only available when those pins have not been provided
    /// yet. After this method has been called once, you can use
    /// [`DRV8825::set_step_mode`] to change the step mode again.
    pub fn enable_mode_control<
        Reset,
        Mode0,
        Mode1,
        Mode2,
        Clk,
        OutputPinError,
    >(
        self,
        reset: Reset,
        mode0: Mode0,
        mode1: Mode1,
        mode2: Mode2,
        step_mode: StepMode32,
        clock: &Clk,
    ) -> Result<
        DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>,
        ModeError<OutputPinError>,
    >
    where
        Reset: OutputPin<Error = OutputPinError>,
        Mode0: OutputPin<Error = OutputPinError>,
        Mode1: OutputPin<Error = OutputPinError>,
        Mode2: OutputPin<Error = OutputPinError>,
        Clk: Clock,
    {
        let mut self_ = DRV8825 {
            enable: self.enable,
            fault: self.fault,
            sleep: self.sleep,
            reset,
            mode0,
            mode1,
            mode2,
            step: self.step,
            dir: self.dir,
        };

        self_.set_step_mode(step_mode, clock)?;

        Ok(self_)
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> SetStepMode
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Reset: OutputPin<Error = OutputPinError>,
    Mode0: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
{
    type Error = ModeError<OutputPinError>;
    type StepMode = StepMode32;

    /// Sets the step mode
    ///
    /// This method is only available, if all the pins required for setting the
    /// step mode have been provided using [`DRV8825::enable_mode_control`].
    fn set_step_mode<Clk: Clock>(
        &mut self,
        step_mode: StepMode32,
        clock: &Clk,
    ) -> Result<(), Self::Error> {
        // 7.6 Timing Requirements (page 7)
        // https://www.ti.com/lit/ds/symlink/drv8825.pdf
        const SETUP_TIME: Nanoseconds = Nanoseconds(650);

        // Reset the device's internal logic and disable the h-bridge drivers.
        self.reset
            .try_set_low()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Set mode signals.
        let (mode0, mode1, mode2) = step_mode_to_signals(&step_mode);
        self.mode0
            .try_set_state(mode0)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.mode1
            .try_set_state(mode1)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.mode2
            .try_set_state(mode2)
            .map_err(|err| ModeError::OutputPin(err))?;

        // Need to wait for the MODEx input setup time.
        clock.new_timer(SETUP_TIME).start()?.wait()?;

        // Re-enable the h-bridge drivers using the new configuration.
        self.reset
            .try_set_high()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Now the mode pins need to stay as they are for the MODEx input hold
        // time, for the settings to take effect.
        clock.new_timer(SETUP_TIME).start()?.wait()?;

        Ok(())
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> DirTrait
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Dir: OutputPin<Error = OutputPinError>,
{
    // 7.6 Timing Requirements (page 7)
    // https://www.ti.com/lit/ds/symlink/drv8825.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds(650);

    type Dir = Dir;
    type Error = OutputPinError;

    fn dir(&mut self) -> &mut Self::Dir {
        &mut self.dir
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> StepTrait
    for DRV8825<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Step: OutputPin<Error = OutputPinError>,
{
    // 7.6 Timing Requirements (page 7)
    // https://www.ti.com/lit/ds/symlink/drv8825.pdf
    const PULSE_LENGTH: Nanoseconds = Nanoseconds(1900);

    type Step = Step;
    type Error = OutputPinError;

    fn step(&mut self) -> &mut Self::Step {
        &mut self.step
    }
}

/// Provides the pin signals for the given step mode
fn step_mode_to_signals(
    step_mode: &StepMode32,
) -> (PinState, PinState, PinState) {
    use PinState::*;
    use StepMode32::*;

    match step_mode {
        Full => (Low, Low, Low),
        M2 => (High, Low, Low),
        M4 => (Low, High, Low),
        M8 => (High, High, Low),
        M16 => (Low, Low, High),
        M32 => (High, High, High),
    }
}
