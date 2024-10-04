use crate::chrono::Chrono;
use core::mem::size_of;
use embedded_hal::digital::{OutputPin, PinState};

pub fn run<C: Chrono, P: OutputPin>(
    p: &Parameters,
    ips: &InputPeripherals<C>,
    ops: &mut OutputPeripherals<P>,
) -> ! {
    loop {
        let inputs = read_inputs(&ips);
        let outputs = advance(p, inputs);
        apply_outputs(outputs, ops)
    }
}

pub struct Inputs {
    pub ms: u32,
}

pub struct Outputs {
    // Each bit is the state of a led
    pub leds: u8,
}

pub struct Parameters {
    pub period_ms: u32,
}

pub fn advance(p: &Parameters, inputs: Inputs) -> Outputs {
    let period = p.period_ms;
    let nb_leds = 8 * size_of::<u8>() as u32;
    // led_idx is the number of the led that should be on
    let led_idx = (nb_leds * (inputs.ms % period)) / period;
    let leds: u8 = 1 << led_idx;
    Outputs { leds }
}

pub struct InputPeripherals<T: Chrono> {
    pub chrono: T,
}

fn read_inputs<T: Chrono>(ps: &InputPeripherals<T>) -> Inputs {
    let ms = ps.chrono.millis();
    Inputs { ms }
}

pub struct OutputPeripherals<T: OutputPin> {
    pub ps: [T; 8],
}

fn apply_outputs<T: OutputPin>(outputs: Outputs, pins: &mut OutputPeripherals<T>) -> () {
    // Set every pin by iterating on the pins list
    for (led_idx, pin) in pins.ps.iter_mut().enumerate() {
        let led_on = outputs.leds >> led_idx & 0x1 != 0;
        pin.set_state(PinState::from(led_on)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outputs() {
        struct TestCase {
            ms: u32,
            expected: u8,
        }
        let cases = [
            TestCase {
                ms: 50,
                expected: 1,
            },
            TestCase {
                ms: 950,
                expected: 128,
            },
            TestCase {
                ms: 300,
                expected: 4,
            },
        ];
        let p = Parameters { period_ms: 1000 };
        for tc in cases {
            let inputs = Inputs { ms: tc.ms };
            let outputs = advance(&p, inputs);
            assert_eq!(tc.expected, outputs.leds, "wrong led pattern")
        }
    }
}
