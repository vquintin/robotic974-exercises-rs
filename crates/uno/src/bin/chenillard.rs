#![no_std]
#![no_main]

use panic_halt as _;

const PARAMS: exercises::chenillard::Parameters = exercises::chenillard::Parameters {
    // The chenillard loops every second
    period_ms: 1000,
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);

    let mut my_pins = exercises::chenillard::OutputPeripherals {
        ps: [
            // Every pin has a different type thus a call to downgrade is
            // necessary to give them the same type and put them in a list
            pins.d6.downgrade(),
            pins.d7.downgrade(),
            pins.d8.downgrade(),
            pins.d9.downgrade(),
            pins.d10.downgrade(),
            pins.d11.downgrade(),
            pins.d12.downgrade(),
            pins.d13.downgrade(),
        ]
        // Turn every pin from the list into an output pin
        .map(|p| p.into_output()),
    };

    let ip = exercises::chenillard::InputPeripherals {
        // Use "millis" based on timer 0
        chrono: uno_helper::timer_0::Chrono0::new(dp.TC0),
    };

    unsafe { avr_device::interrupt::enable() }

    // start the chenillard lib using the arduino specific peripherals
    exercises::chenillard::run(&PARAMS, &ip, &mut my_pins);
}
