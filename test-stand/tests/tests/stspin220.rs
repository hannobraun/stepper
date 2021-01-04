#![no_std]
#![no_main]

use test_stand as _;

use test_stand::{
    lpc8xx_hal::{
        self,
        gpio::{
            self,
            direction::{Input, Output},
            GpioPin,
        },
        mrt::{self, MRT0},
        pins::{
            PIO0_0, PIO0_1, PIO0_16, PIO0_17, PIO0_18, PIO0_19, PIO0_20, PIO1_0,
        },
    },
    rotary_encoder_hal::Rotary,
    step_dir::{drivers::stspin220::STSPIN220, Direction, Driver, StepMode256},
    test_step,
};

struct Context {
    driver: Driver<
        STSPIN220<
            (),
            GpioPin<PIO0_16, Output>,
            GpioPin<PIO0_17, Output>,
            GpioPin<PIO0_18, Output>,
            GpioPin<PIO0_19, Output>,
            GpioPin<PIO0_20, Output>,
        >,
    >,
    timer: mrt::Channel<MRT0>,
    rotary: Rotary<GpioPin<PIO0_0, Input>, GpioPin<PIO0_1, Input>>,
    debug_signal: GpioPin<PIO1_0, Output>,
}

#[defmt_test::tests]
mod tests {
    #[init]
    fn init() -> super::Context {
        use super::{
            gpio, lpc8xx_hal, mrt, Direction, Driver, Rotary, StepMode256,
            STSPIN220,
        };

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

        let debug_signal = p
            .pins
            .pio1_0
            .into_output_pin(gpio.tokens.pio1_0, gpio::Level::Low);

        let mut timer = mrt.mrt0;

        timer.start(mrt::MAX_VALUE);
        let driver = Driver::from_inner(STSPIN220::new())
            .enable_step_control(step_mode3)
            .enable_direction_control(dir_mode4, Direction::Forward, &timer)
            .unwrap()
            .enable_step_mode_control(
                (standby_reset, mode1, mode2),
                StepMode256::Full,
                &timer,
            )
            .unwrap();

        let rotary = Rotary::new(rotary_a, rotary_b);

        super::Context {
            driver,
            timer,
            rotary,
            debug_signal,
        }
    }

    #[test]
    fn test_step(cx: &mut super::Context) {
        super::test_step(
            &mut cx.driver,
            &mut cx.timer,
            &mut cx.rotary,
            &mut cx.debug_signal,
        );
    }
}
