#![no_std]
#![no_main]

use panic_halt as _;
use rand::{rngs::SmallRng, SeedableRng};

const PARAMS: exercises::dice::Parameters = exercises::dice::Parameters {
    first_blink_duration_ms: 100,
    last_blink_duration_ms: 400,
    nb_blinks: 10,
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);

    let mut my_pins = exercises::dice::OutputPeripherals {
        leds: [
            pins.d6.downgrade(),
            pins.d7.downgrade(),
            pins.d8.downgrade(),
            pins.d9.downgrade(),
            pins.d10.downgrade(),
        ]
        .map(|p| p.into_output()),
    };

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let adc_a0 = pins.a0.into_analog_input(&mut adc);

    // Blocking read on init to get a seed for the RNG
    let seed = adc_a0.analog_read(&mut adc);

    let mut ip = exercises::dice::InputPeripherals {
        chrono: uno_helper::timer_0::Chrono0::new(dp.TC0),
        button: pins.d11.into_pull_up_input(),
        rng: SmallRng::seed_from_u64(seed as u64),
    };

    unsafe { avr_device::interrupt::enable() }

    exercises::dice::run(&PARAMS, &mut ip, &mut my_pins);
}
