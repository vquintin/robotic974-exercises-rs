#![no_std]
#![no_main]
#![feature(const_option)]

use fixed::types::{I3F13, U1F15, U24F8};
use panic_halt as _;

// 16MHz, not sure how to get that from arduino hal
const CPU_FREQUENCY: U24F8 = U24F8::lit("16_000_000");

// PWM runs at 50Hz
const PWM_FREQUENCY: U24F8 = U24F8::lit("50");

// 50Hz => 20_000us
const PWM_PERIOD_US: u32 = U24F8::lit("1_000_000").to_bits() / PWM_FREQUENCY.to_bits();

const PARAMS: exercises::wiper::Parameters = exercises::wiper::Parameters {
    // PI / 4 rad.s^-1 => half a turn in 4 seconds
    absolute_speed: U1F15::lit("0.7853981633"),
    // -PI / 2 or -90 deg
    min_position: I3F13::lit("-1.57079632679"),
    min_position_us: 500,
    max_position: I3F13::lit("1.57079632679"),
    max_position_us: 2500,
    pwm_period_us: PWM_PERIOD_US,
};

// Compute prescaler and icr1 at compile time
// compute_timer_params does not end up in binary, only the computed constants
const PRECISE_TIMER_PARAMS: uno_lib::timer_1::PreciseTimerParams =
    uno_lib::timer_1::compute_timer_params(
        CPU_FREQUENCY,
        PWM_FREQUENCY,
        // I want it to run at =exactly* 50Hz or fail the compilation
        U24F8::lit("0"),
    )
    .unwrap();

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);

    pins.d9.into_output();

    let s = uno_helper::precise_pwm::new_precise_pwm(dp.TC1, PRECISE_TIMER_PARAMS);

    let mut my_pins = exercises::wiper::OutputPeripherals {
        servo: s,
        serial: arduino_hal::default_serial!(dp, pins, 57600),
    };

    let mut ip = exercises::wiper::InputPeripherals {
        chrono: uno_helper::timer_0::Chrono0::new(dp.TC0),
        button: pins.d11.into_pull_up_input(),
    };

    unsafe { avr_device::interrupt::enable() }

    exercises::wiper::run(&PARAMS, &mut ip, &mut my_pins);
}
