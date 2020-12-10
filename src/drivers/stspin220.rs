//! STSPIN220 Driver
//!
//! Platform-agnostic driver for the STSPIN220 stepper motor driver. This module
//! can be used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! The entry point to this module is the [`STSPIN220`] struct.
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
//!     drivers::stspin220::STSPIN220,
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
//! # let step_mode3 = Pin;
//! # let dir_mode4 = Pin;
//! # let mut clock = Clock(std::time::Instant::now());
//! #
//! // You need to acquire the GPIO pins connected to the STEP/MODE3 and
//! // DIR/MODE4 signals. How you do this depends on your target platform. All
//! // the driver cares about is that they implement
//! // `embedded_hal::digital::OutputPin`. You also need an implementation of
//! // `embedded_hal::blocking::DelayUs`.
//!
//! // Create driver API from STEP/MODE3 and DIR/MODE4 pins.
//! let mut driver = Driver::new(
//!     STSPIN220::from_step_dir_pins(step_mode3, dir_mode4)
//! );
//!
//! // Rotate stepper motor by a few steps.
//! for _ in 0 .. 5 {
//!     let timer = clock.new_timer(STEP_DELAY).start()?;
//!     driver.step(Direction::Forward, &clock)?;
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
use embedded_time::{
    duration::{Microseconds, Nanoseconds},
    Clock,
};

use crate::{
    traits::{Dir, SetStepMode, Step},
    ModeError, StepMode256,
};

/// The STSPIN220 driver API
///
/// You can create an instance of this struct by calling
/// [`STSPIN220::from_step_dir_pins`]. See [module documentation] for a full
/// example that uses this API.
///
/// [module documentation]: index.html
pub struct STSPIN220<
    EnableFault,
    StandbyReset,
    Mode1,
    Mode2,
    StepMode3,
    DirMode4,
> {
    enable_fault: EnableFault,
    standby_reset: StandbyReset,
    mode1: Mode1,
    mode2: Mode2,
    step_mode3: StepMode3,
    dir_mode4: DirMode4,
}

impl<StepMode3, DirMode4> STSPIN220<(), (), (), (), StepMode3, DirMode4> {
    /// Create a new instance of `STSPIN220`
    ///
    /// Creates an instance of this struct from just the STEP/MODE3 and
    /// DIR/MODE4 pins. It expects the types that represent those pins to
    /// implement [`OutputPin`].
    ///
    /// The resulting instance can be used to step the motor using
    /// [`STSPIN220::step`]. All other capabilities of the STSPIN220, like
    /// the power-up sequence, selecting a step mode, or controlling the power
    /// state, explicitly enabled, or managed externally.
    ///
    /// To enable additional capabilities, see
    /// [`STSPIN220::enable_mode_control`].
    pub fn from_step_dir_pins<Error>(
        step_mode3: StepMode3,
        dir_mode4: DirMode4,
    ) -> Self
    where
        StepMode3: OutputPin<Error = Error>,
        DirMode4: OutputPin<Error = Error>,
    {
        Self {
            enable_fault: (),
            standby_reset: (),
            mode1: (),
            mode2: (),
            step_mode3,
            dir_mode4,
        }
    }
}

impl<EnableFault, StepMode3, DirMode4>
    STSPIN220<EnableFault, (), (), (), StepMode3, DirMode4>
{
    /// Enables support for step mode control and sets the initial step mode
    ///
    /// Consumes this instance of `STSPIN220` and returns another instance that
    /// has support for controlling the step mode. Requires the additional pins
    /// for doing so, namely STBY/RESET, MODE1, and MODE2. It expects the types
    /// that represent those pins to implement [`OutputPin`].
    ///
    /// This method is only available when those pins have not been provided
    /// yet. After this method has been called once, you can use
    /// [`STSPIN220::set_step_mode`] to change the step mode again.
    pub fn enable_mode_control<
        StandbyReset,
        Mode1,
        Mode2,
        Clk,
        OutputPinError,
    >(
        self,
        standby_reset: StandbyReset,
        mode1: Mode1,
        mode2: Mode2,
        step_mode: StepMode256,
        clock: &Clk,
    ) -> Result<
        STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>,
        ModeError<OutputPinError>,
    >
    where
        StandbyReset: OutputPin<Error = OutputPinError>,
        Mode1: OutputPin<Error = OutputPinError>,
        Mode2: OutputPin<Error = OutputPinError>,
        StepMode3: OutputPin<Error = OutputPinError>,
        DirMode4: OutputPin<Error = OutputPinError>,
        Clk: Clock,
    {
        let mut self_ = STSPIN220 {
            enable_fault: self.enable_fault,
            standby_reset,
            mode1,
            mode2,
            step_mode3: self.step_mode3,
            dir_mode4: self.dir_mode4,
        };

        self_.set_step_mode(step_mode, clock)?;

        Ok(self_)
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > SetStepMode
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
where
    StandbyReset: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
    StepMode3: OutputPin<Error = OutputPinError>,
    DirMode4: OutputPin<Error = OutputPinError>,
{
    type Error = ModeError<OutputPinError>;

    type StepMode = StepMode256;

    /// Sets the step mode
    ///
    /// This method is only available, if all the pins required for setting the
    /// step mode have been provided using [`STSPIN220::enable_mode_control`].
    fn set_step_mode<Clk: Clock>(
        &mut self,
        step_mode: Self::StepMode,
        clock: &Clk,
    ) -> Result<(), Self::Error> {
        const MODE_SETUP_TIME: Microseconds = Microseconds(1);
        const MODE_HOLD_TIME: Microseconds = Microseconds(100);

        // Force driver into standby mode.
        self.standby_reset
            .try_set_low()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Set mode signals.
        let (mode1, mode2, mode3, mode4) = step_mode_to_signals(&step_mode);
        self.mode1
            .try_set_state(mode1)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.mode2
            .try_set_state(mode2)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.step_mode3
            .try_set_state(mode3)
            .map_err(|err| ModeError::OutputPin(err))?;
        self.dir_mode4
            .try_set_state(mode4)
            .map_err(|err| ModeError::OutputPin(err))?;

        // Need to wait for the MODEx input setup time.
        clock.new_timer(MODE_SETUP_TIME).start()?.wait()?;

        // Leave standby mode.
        self.standby_reset
            .try_set_high()
            .map_err(|err| ModeError::OutputPin(err))?;

        // Now the mode pins need to stay as they are for the MODEx input hold
        // time, for the settings to take effect.
        clock.new_timer(MODE_HOLD_TIME).start()?.wait()?;

        Ok(())
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > Dir
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
where
    DirMode4: OutputPin<Error = OutputPinError>,
{
    const SETUP_TIME: Nanoseconds = Nanoseconds(100);

    type Dir = DirMode4;
    type Error = OutputPinError;

    fn dir(&mut self) -> &mut Self::Dir {
        &mut self.dir_mode4
    }
}

impl<
        EnableFault,
        StandbyReset,
        Mode1,
        Mode2,
        StepMode3,
        DirMode4,
        OutputPinError,
    > Step
    for STSPIN220<EnableFault, StandbyReset, Mode1, Mode2, StepMode3, DirMode4>
where
    StepMode3: OutputPin<Error = OutputPinError>,
{
    const PULSE_LENGTH: Nanoseconds = Nanoseconds(100);

    type Step = StepMode3;
    type Error = OutputPinError;

    fn step(&mut self) -> &mut Self::Step {
        &mut self.step_mode3
    }
}

/// Provides the pin signals for the given step mode
pub fn step_mode_to_signals(
    step_mode: &StepMode256,
) -> (PinState, PinState, PinState, PinState) {
    use PinState::*;
    use StepMode256::*;
    match step_mode {
        Full => (Low, Low, Low, Low),
        M2 => (High, Low, High, Low),
        M4 => (Low, High, Low, High),
        M8 => (High, High, High, Low),
        M16 => (High, High, High, High),
        M32 => (Low, High, Low, Low),
        M64 => (High, High, Low, High),
        M128 => (High, Low, Low, Low),
        M256 => (High, High, Low, Low),
    }
}
