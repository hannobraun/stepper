#![no_std]
#![no_main]

use test_stand as _;

use core::fmt::Debug;

use test_stand::{
    lpc8xx_hal::{
        self,
        gpio::{self, direction::Output, GpioPin},
        mrt::{self, MRT0},
        pins::{PIO0_16, PIO0_17, PIO0_18, PIO0_19, PIO0_20},
    },
    step_dir::{
        embedded_hal::timer,
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
}

#[defmt_test::tests]
mod tests {
    #[init]
    fn init() -> super::Context {
        use super::{gpio, lpc8xx_hal, mrt, StepMode256, STSPIN220};

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

        super::Context { driver, timer }
    }

    #[test]
    fn test_step(cx: &mut super::Context) {
        super::test_step(&mut cx.driver, &mut cx.timer);
    }
}

fn test_step<Driver, Timer>(driver: &mut Driver, timer: &mut Timer)
where
    Driver: Step,
    Driver::Error: Debug,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
{
    verify_steps(driver, timer, Dir::Forward);
    verify_steps(driver, timer, Dir::Backward);
}

fn verify_steps<Driver, Timer>(
    driver: &mut Driver,
    timer: &mut Timer,
    direction: Dir,
) where
    Driver: Step,
    Driver::Error: Debug,
    Timer: timer::CountDown<Time = u32> + Clock,
    Timer::Error: Debug,
{
    const STEP_DELAY: Microseconds = Microseconds(10_000);

    for _ in 0..20 {
        timer.try_start(mrt::MAX_VALUE).unwrap();
        let step_timer = timer.new_timer(STEP_DELAY).start().unwrap();
        driver.step(direction, timer).unwrap();
        step_timer.wait().unwrap();
    }
}
