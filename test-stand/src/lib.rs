#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

// Re-export dependencies, so test suite can use them.
pub extern crate defmt;
pub extern crate lpc8xx_hal;
pub extern crate rotary_encoder_hal;
pub extern crate step_dir;

use core::fmt::Debug;

use lpc8xx_hal::{cortex_m::asm, mrt};
use rotary_encoder_hal::{Direction, Rotary};
use step_dir::{
    embedded_hal::{
        digital::{InputPin, OutputPin},
        timer,
    },
    embedded_time::{duration::Microseconds, Clock},
    Dir, SetStepMode, Step, StepMode as _,
};

/// Causes probe-run to exit with exit code 0
pub fn exit() -> ! {
    loop {
        asm::bkpt();
    }
}

pub fn test_step<Driver, Timer, A, B, DebugSignal>(
    driver: &mut Driver,
    timer: &mut Timer,
    rotary: &mut Rotary<A, B>,
    debug_signal: &mut DebugSignal,
) where
    Driver: Step,
    Driver::Error: Debug,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
    A: InputPin,
    A::Error: Debug,
    B: InputPin,
    B::Error: Debug,
    DebugSignal: OutputPin,
    DebugSignal::Error: Debug,
{
    verify_steps(driver, timer, rotary, 1, Dir::Forward, debug_signal);
    verify_steps(driver, timer, rotary, 1, Dir::Backward, debug_signal);
}

pub fn test_set_step_mode<Driver, Timer, A, B, DebugSignal>(
    driver: &mut Driver,
    timer: &mut Timer,
    rotary: &mut Rotary<A, B>,
    debug_signal: &mut DebugSignal,
) where
    Driver: Step + SetStepMode,
    <Driver as Step>::Error: Debug,
    <Driver as SetStepMode>::Error: Debug,
    <Driver as SetStepMode>::StepMode: Copy,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
    A: InputPin,
    A::Error: Debug,
    B: InputPin,
    B::Error: Debug,
    DebugSignal: OutputPin,
    DebugSignal::Error: Debug,
{
    for mode in Driver::StepMode::iter() {
        driver.set_step_mode(mode, timer).unwrap();
        verify_steps(
            driver,
            timer,
            rotary,
            mode.into(),
            Dir::Forward,
            debug_signal,
        );
    }
}

pub fn verify_steps<Driver, Timer, A, B, DebugSignal>(
    driver: &mut Driver,
    timer: &mut Timer,
    rotary: &mut Rotary<A, B>,
    microsteps: u16,
    direction: Dir,
    debug_signal: &mut DebugSignal,
) where
    Driver: Step,
    Driver::Error: Debug,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
    A: InputPin,
    A::Error: Debug,
    B: InputPin,
    B::Error: Debug,
    DebugSignal: OutputPin,
    DebugSignal::Error: Debug,
{
    const STEP_DELAY: Microseconds = Microseconds(10_000);

    // Stepper motor has 200 steps per revolution, encoder has 20 counts.
    const STEPS_PER_COUNT: u32 = 10;

    // Discard the first direction read by the rotary encoder. It starts out
    // with an initial state that is constant, i.e. doesn't reflect the actual
    // initial state. Unless the actual state happens to be the same by
    // accident, it will mistake its initial reading for a first movement.
    let _ = rotary.update().unwrap();

    // Set test output signal. This is useful when debugging with a logic
    // analyzer, as it demarcates the initial setup movement and the actual test
    // movement.
    debug_signal.try_set_high().unwrap();

    // Depending on the initial position of the rotary encoder's magnet, we
    // might not read the number of encoder counts correctly.  If we start at a
    // position where the encoder would count, then we'll end up at such a
    // position again, after we're done rotating. That last count might or might
    // not be detected.
    //
    // We can solve that by making sure we start out right in the middle between
    // two such positions. Then we'll also end up in the middle at the end of
    // the movement and can be sure all counts are detected.
    //
    // Here we step the motor until we read a count, then move half the number
    // of steps that make up a count, to get us to a desired middle position.
    while step(driver, timer, rotary, STEP_DELAY, direction, false) == 0 {}
    for _ in 0..STEPS_PER_COUNT / 2 {
        step(driver, timer, rotary, STEP_DELAY, direction, false);
    }

    // Setup movement is over. Lower test signal.
    debug_signal.try_set_low().unwrap();

    let delay = STEP_DELAY / microsteps as u32;
    let steps = 20;
    let counts_expected = steps / STEPS_PER_COUNT;

    let mut counts = 0;
    for _ in 0..steps * microsteps as u32 {
        counts += step(driver, timer, rotary, delay, direction, true);
    }

    defmt::info!(
        "Encoder counts expected: {:?}; measured: {:?}",
        counts_expected,
        counts
    );
    assert_eq!(counts_expected, counts);
}

pub fn step<Driver, Timer, A, B>(
    driver: &mut Driver,
    timer: &mut Timer,
    rotary: &mut Rotary<A, B>,
    delay: Microseconds,
    direction: Dir,
    check_direction: bool,
) -> u32
where
    Driver: Step,
    Driver::Error: Debug,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
    A: InputPin,
    A::Error: Debug,
    B: InputPin,
    B::Error: Debug,
{
    let expected_direction = match direction {
        Dir::Forward => Direction::Clockwise,
        Dir::Backward => Direction::CounterClockwise,
    };

    timer.try_start(mrt::MAX_VALUE).unwrap();
    let step_timer = timer.new_timer(delay).start().unwrap();
    driver.step(direction, timer).unwrap();

    let mut counts = 0;

    loop {
        match rotary.update().unwrap() {
            Direction::None => {}
            direction if direction == expected_direction => {
                counts += 1;
            }
            direction => {
                // Depending on initial conditions, we can get an unexpected
                // direction reading here. To prevent this during the test, the
                // initial conditions are set up in a controlled way. The
                // `check_direction` argument gives the caller the opportunity
                // to disable this check, while this is done.
                if check_direction {
                    defmt::panic!("Unexpected direction: {:?}", direction);
                }
            }
        }

        if step_timer.is_expired().unwrap() {
            break;
        }
    }

    counts
}
