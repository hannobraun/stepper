#![no_std]
#![no_main]

use test_stand as _;

use core::fmt::Debug;

use test_stand::{
    lpc8xx_hal::{
        self,
        gpio::{
            self,
            direction::{Input, Output},
            GpioPin,
        },
        mrt::{self, MRT0},
        pins::{PIO0_0, PIO0_1, PIO0_16, PIO0_17, PIO0_18, PIO0_19, PIO0_20},
    },
    rotary_encoder_hal::{Direction, Rotary},
    step_dir::{
        embedded_hal::{digital::InputPin, timer},
        embedded_time::{clock::Clock, duration::Microseconds},
        stspin220::STSPIN220,
        Dir, Step, StepMode256,
    },
};

struct Context {
    driver: STSPIN220<
        (),
        GpioPin<PIO0_16, Output>,
        GpioPin<PIO0_17, Output>,
        GpioPin<PIO0_18, Output>,
        GpioPin<PIO0_19, Output>,
        GpioPin<PIO0_20, Output>,
    >,
    timer: mrt::Channel<MRT0>,
    rotary: Rotary<GpioPin<PIO0_0, Input>, GpioPin<PIO0_1, Input>>,
}

#[defmt_test::tests]
mod tests {
    #[init]
    fn init() -> super::Context {
        use super::{gpio, lpc8xx_hal, mrt, Rotary, StepMode256, STSPIN220};

        let p = lpc8xx_hal::Peripherals::take().unwrap();

        let mut syscon = p.SYSCON.split();
        let gpio = p.GPIO.enable(&mut syscon.handle);
        let mrt = p.MRT0.split(&mut syscon.handle);

        let standby_reset = p
            .pins
            .pio0_16
            .into_output_pin(gpio.tokens.pio0_16, gpio::Level::Low);
        let mode1 = p
            .pins
            .pio0_17
            .into_output_pin(gpio.tokens.pio0_17, gpio::Level::Low);
        let mode2 = p
            .pins
            .pio0_18
            .into_output_pin(gpio.tokens.pio0_18, gpio::Level::Low);
        let step_mode3 = p
            .pins
            .pio0_19
            .into_output_pin(gpio.tokens.pio0_19, gpio::Level::Low);
        let dir_mode4 = p
            .pins
            .pio0_20
            .into_output_pin(gpio.tokens.pio0_20, gpio::Level::Low);

        let rotary_a = p.pins.pio0_0.into_input_pin(gpio.tokens.pio0_0);
        let rotary_b = p.pins.pio0_1.into_input_pin(gpio.tokens.pio0_1);

        let mut timer = mrt.mrt0;

        timer.start(mrt::MAX_VALUE);
        let driver = STSPIN220::from_step_dir_pins(step_mode3, dir_mode4)
            .enable_mode_control(
                standby_reset,
                mode1,
                mode2,
                StepMode256::Full,
                &timer,
            )
            .unwrap();

        let rotary = Rotary::new(rotary_a, rotary_b);

        super::Context {
            driver,
            timer,
            rotary,
        }
    }

    #[test]
    fn test_step(cx: &mut super::Context) {
        super::test_step(&mut cx.driver, &mut cx.timer, &mut cx.rotary);
    }
}

fn test_step<Driver, Timer, A, B>(
    driver: &mut Driver,
    timer: &mut Timer,
    rotary: &mut Rotary<A, B>,
) where
    Driver: Step,
    Driver::Error: Debug,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
    A: InputPin,
    A::Error: Debug,
    B: InputPin,
    B::Error: Debug,
{
    verify_steps(driver, timer, rotary, Dir::Forward);
    verify_steps(driver, timer, rotary, Dir::Backward);
}

fn verify_steps<Driver, Timer, A, B>(
    driver: &mut Driver,
    timer: &mut Timer,
    rotary: &mut Rotary<A, B>,
    direction: Dir,
) where
    Driver: Step,
    Driver::Error: Debug,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
    A: InputPin,
    A::Error: Debug,
    B: InputPin,
    B::Error: Debug,
{
    const STEP_DELAY: Microseconds = Microseconds(10_000);

    // Stepper motor has 200 steps per revolution, encoder has 20 counts.
    const STEPS_PER_COUNT: u32 = 10;

    // Discard the first direction read by the rotary encoder. It starts out
    // with an initial state that is constant, i.e. doesn't reflect the actual
    // initial state. Unless the actual state happens to be the same by
    // accident, it will mistake its initial reading for a first movement.
    let _ = rotary.update().unwrap();

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

    let steps = 20;
    let counts_expected = steps / STEPS_PER_COUNT;

    let mut counts = 0;
    for _ in 0..steps {
        counts += step(driver, timer, rotary, STEP_DELAY, direction, true);
    }

    defmt::info!(
        "Encoder counts expected: {:?}; measured: {:?}",
        counts_expected,
        counts
    );
    assert_eq!(counts_expected, counts);
}

fn step<Driver, Timer, A, B>(
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
