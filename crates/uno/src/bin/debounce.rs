#![no_std]
#![no_main]

use fixed::types::U1F15;
use uno_helper::timer_0;

const PARAMS: exercises::debounce::Parameters = exercises::debounce::Parameters {
    low_pass_params: exercises::low_pass::FixedLowPassParams {
        // 1 / 0.001 = 100
        // The button is debounced on 100ms
        lambda: U1F15::lit("1e-2"),
    },
    // A 0.95 average for the left button makes us consider it pressed
    threshold: U1F15::lit("95e-2"),
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);

    let mut output_peripherals = exercises::debounce::OutputPeripherals {
        serial: arduino_hal::default_serial!(dp, pins, 57600),
    };

    let mut input_peripherals = exercises::debounce::InputPeripherals {
        chrono: timer_0::Chrono0::new(dp.TC0),
        left_button: pins.d10.downgrade().into_pull_up_input(),
        right_button: pins.d11.downgrade().into_pull_up_input(),
    };

    unsafe { avr_device::interrupt::enable() }

    exercises::debounce::run(&PARAMS, &mut input_peripherals, &mut output_peripherals);
}
