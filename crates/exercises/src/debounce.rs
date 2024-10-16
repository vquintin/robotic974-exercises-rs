use crate::chrono::Chrono;
use crate::low_pass::{FixedLowPassParams, FixedLowPassState};
use embedded_hal::digital::InputPin;
use fixed::types::U1F15;

pub fn run<C: Chrono, IP: InputPin, S: ufmt::uWrite>(
    p: &Parameters,
    ips: &mut InputPeripherals<C, IP>,
    ops: &mut OutputPeripherals<S>,
) -> ! {
    let mut state = State {
        left_button: FixedLowPassState::new(),
        right_button: false,

        left_presses: 0,
        right_presses: 0,
    };
    loop {
        let inputs = read_inputs(ips);
        let (new_state, outputs) = advance(p, state, inputs);
        state = new_state;
        apply_outputs(outputs, ops)
    }
}

pub struct Inputs {
    pub ms: u32,
    pub left_button: bool,
    pub right_button: bool,
}

pub struct Outputs {
    pub left_button_val: U1F15,
    pub left_presses: u32,
    pub right_presses: u32,
}

pub struct Parameters {
    pub low_pass_params: FixedLowPassParams,
    // Threshold between 0 and 1 at which we consider the left button as pressed
    pub threshold: U1F15,
}

pub struct State {
    // The left button in a moving average between 0 and 1 instead of a bool
    pub left_button: FixedLowPassState,
    pub right_button: bool,

    pub left_presses: u32,
    pub right_presses: u32,
}

pub fn advance(p: &Parameters, s: State, inputs: Inputs) -> (State, Outputs) {
    let old_left_value = s.left_button.current_val();
    let new_left_button = s.left_button.advance(
        &p.low_pass_params,
        U1F15::unwrapped_from_num(inputs.left_button),
        inputs.ms,
    );
    let new_left_value = new_left_button.current_val();
    let left_pressed = new_left_button.current_val() > p.threshold && old_left_value <= p.threshold;
    let right_pressed = inputs.right_button && !s.right_button;
    let new_state = State {
        left_button: new_left_button,
        right_button: inputs.right_button,
        left_presses: s.left_presses + left_pressed as u32,
        right_presses: s.right_presses + right_pressed as u32,
    };
    let outputs = Outputs {
        left_button_val: new_left_value,
        left_presses: new_state.left_presses,
        right_presses: new_state.right_presses,
    };
    (new_state, outputs)
}

pub struct InputPeripherals<T: Chrono, IP: InputPin> {
    pub chrono: T,
    pub left_button: IP,
    pub right_button: IP,
}

fn read_inputs<T: Chrono, IP: InputPin>(ps: &mut InputPeripherals<T, IP>) -> Inputs {
    Inputs {
        ms: ps.chrono.millis(),
        left_button: ps.left_button.is_low().unwrap(),
        right_button: ps.right_button.is_low().unwrap(),
    }
}

pub struct OutputPeripherals<S> {
    pub serial: S,
}

fn apply_outputs<S: ufmt::uWrite>(outputs: Outputs, pins: &mut OutputPeripherals<S>) -> () {
    ufmt::uwrite!(
        &mut pins.serial,
        "{}\t{}\t{}\n",
        outputs.left_button_val.to_bits(),
        outputs.left_presses,
        outputs.right_presses,
    )
    .unwrap_or(());
}
