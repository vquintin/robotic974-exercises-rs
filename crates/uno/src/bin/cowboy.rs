#![no_std]
#![no_main]

use uno_helper::timer_0;

const PARAMS: exercises::cowboy::Parameters = exercises::cowboy::Parameters {
    nb_blinks: 3,
    blink_duration_ms: 3_000,
    show_winner_duration: 5_000,
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);

    // Use D12 and D13 as the leds to show the results
    let mut output_peripherals = exercises::cowboy::OutputPeripherals {
        left_led: pins.d12.downgrade().into_output(),
        right_led: pins.d13.downgrade().into_output(),
    };

    // Use D10 and D11 as the button inputs
    let mut input_peripherals = exercises::cowboy::InputPeripherals {
        chrono: timer_0::Chrono0::new(dp.TC0),
        left_button: pins.d10.downgrade().into_pull_up_input(),
        right_button: pins.d11.downgrade().into_pull_up_input(),
    };

    unsafe { avr_device::interrupt::enable() }

    exercises::cowboy::run(&PARAMS, &mut input_peripherals, &mut output_peripherals);
}
