use crate::adc::Adc;
use embedded_hal::digital::{OutputPin, PinState};
use fixed::types::{U1F15, U4F12};

// Number of leds + 1
const NINE: U4F12 = U4F12::lit("9");

pub fn run<PIN, A: Adc<PIN>, P: OutputPin, S: ufmt::uWrite>(
    ips: &mut InputPeripherals<PIN, A>,
    ops: &mut OutputPeripherals<P, S>,
) -> ! {
    let mut state = State {
        current_level: U1F15::ZERO,
    };
    loop {
        let inputs = read_inputs(ips);
        let (new_state, outputs) = advance(state, inputs);
        state = new_state;
        apply_outputs(outputs, ops)
    }
}

pub struct Inputs {
    // between 0 and 1.999..
    pub level: Option<U1F15>,
}

pub struct Outputs {
    // The normalized between 0 and 1 value of the ADC
    // For serial port debug
    pub adc_value: U1F15,
    // The number of should be turned on
    // For serial port debug
    pub nb_lebs: u8,
    // Each bit is the state of a led
    // This is 2^nb_leds - 1
    pub leds: u8,
}

pub struct State {
    // The last know value of the ADC
    // This is used if the ADC is currently still reading, to avoid blocking
    pub current_level: U1F15,
}

pub fn advance(s: State, inputs: Inputs) -> (State, Outputs) {
    // Use the new value, or the old one if we don't have a new value
    let current = inputs.level.unwrap_or(s.current_level);
    let nb_leds: u8 = current.wide_mul(NINE).to_num();
    let leds = 1_u8.checked_shl(nb_leds as u32).map_or(u8::MAX, |n| n - 1);
    (
        State {
            current_level: current,
        },
        Outputs {
            adc_value: current,
            nb_lebs: nb_leds,
            leds,
        },
    )
}

pub struct InputPeripherals<PIN, A: crate::adc::Adc<PIN>> {
    pub adc: A,
    pub pin: PIN,
}

fn read_inputs<PIN, A: crate::adc::Adc<PIN>>(ps: &mut InputPeripherals<PIN, A>) -> Inputs {
    Inputs {
        level: ps.adc.read_nonblocking(&ps.pin).ok(),
    }
}

pub struct OutputPeripherals<T: OutputPin, S> {
    pub ps: [T; 8],
    pub serial: S,
}

fn apply_outputs<T: OutputPin, S: ufmt::uWrite>(
    outputs: Outputs,
    pins: &mut OutputPeripherals<T, S>,
) -> () {
    for (led_idx, pin) in pins.ps.iter_mut().enumerate() {
        pin.set_state(PinState::from(outputs.leds >> led_idx & 0x1 != 0))
            .unwrap()
    }
    ufmt::uwrite!(
        &mut pins.serial,
        "nb_leds={}\tled_byte={}\tadc_value={}\n",
        outputs.nb_lebs,
        outputs.leds,
        outputs.adc_value.to_bits(),
    )
    .unwrap_or(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outputs() {
        struct TestCase {
            level: U1F15,
            expected: u8,
        }
        let cases = [
            TestCase {
                level: U1F15::ZERO,
                expected: 0,
            },
            TestCase {
                level: U1F15::ONE,
                expected: u8::MAX,
            },
            TestCase {
                level: U1F15::lit("7e-1"),
                expected: (1 << 6) - 1,
            },
        ];
        for tc in cases {
            let s: State = State {
                current_level: U1F15::ZERO,
            };
            let inputs = Inputs {
                level: Some(tc.level),
            };
            let (_, outputs) = advance(s, inputs);
            assert_eq!(tc.expected, outputs.leds, "wrong led pattern")
        }
    }
}
