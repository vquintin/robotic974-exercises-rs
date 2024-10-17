#![no_std]
#![no_main]

use arduino_hal::{adc::AdcChannel, hal::Atmega};
use fixed::{traits::ToFixed, types::U10F6};
use panic_halt as _;

// The uno adc values go up to 1023 (10-bit ADC).
const ADC_MAX: U10F6 = U10F6::lit("1023");

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);

    let mut my_pins = exercises::vumetre::OutputPeripherals {
        ps: [
            pins.d6.downgrade(),
            pins.d7.downgrade(),
            pins.d8.downgrade(),
            pins.d9.downgrade(),
            pins.d10.downgrade(),
            pins.d11.downgrade(),
            pins.d12.downgrade(),
            pins.d13.downgrade(),
        ]
        .map(|p| p.into_output()),
        serial: arduino_hal::default_serial!(dp, pins, 57600),
    };

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let adc_a0 = pins.a0.into_analog_input(&mut adc);

    // Use the only adc on the uno, on channel A0
    let mut ip = exercises::vumetre::InputPeripherals {
        adc: AdcWrapper(adc),
        pin: adc_a0,
    };

    unsafe { avr_device::interrupt::enable() }

    exercises::vumetre::run(&mut ip, &mut my_pins);
}

struct AdcWrapper(arduino_hal::adc::Adc);

// Implements Adc trait we declared in this repo to convert adc values in [0, 1023]
// to a "device independant" value on [0, 1], using a fixed point number
// The read from the ADC is non blocking. If the ADC is not ready, we get a "WouldBlock"
// error token
impl<PIN> exercises::adc::Adc<PIN> for AdcWrapper
where
    PIN: AdcChannel<Atmega, arduino_hal::pac::ADC>,
{
    fn read_nonblocking(
        &mut self,
        pin: &PIN,
    ) -> nb::Result<fixed::types::U1F15, core::convert::Infallible> {
        let res = self.0.read_nonblocking(pin);
        res.map(|x| (U10F6::from_num(x).wide_div(ADC_MAX)).to_fixed())
    }
}
