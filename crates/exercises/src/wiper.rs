use embedded_hal::{digital::InputPin, pwm::SetDutyCycle};
use fixed::{
    traits::{FromFixed, ToFixed},
    types::{I16F0, I3F13, U1F15},
};

use crate::chrono::Chrono;

#[inline]
pub fn run<C: Chrono, IP: InputPin, S: embedded_hal::pwm::SetDutyCycle, SERIAL: ufmt::uWrite>(
    params: &Parameters,
    ips: &mut InputPeripherals<C, IP>,
    ops: &mut OutputPeripherals<S, SERIAL>,
) -> ! {
    let mut state = State {
        position: params.min_position,
        ts: 0,
    };
    loop {
        let inputs = read_inputs(ips);
        // Generate a new state of the game instead of mutating the current state
        let (new_state, outputs) = update_state(params, state, inputs);
        state = new_state;
        apply_outputs(params, outputs, ops)
    }
}

struct Inputs {
    ms: u32,
    button_pressed: bool,
}

struct Outputs {
    position: I3F13,
    button: bool,
}

struct State {
    position: I3F13,
    ts: u32,
}

pub struct Parameters {
    pub absolute_speed: U1F15,
    pub min_position: I3F13,
    pub min_position_us: u16,
    pub max_position: I3F13,
    pub max_position_us: u16,
    pub pwm_period_us: u32,
}

fn update_state(p: &Parameters, s: State, input: Inputs) -> (State, Outputs) {
    let direction: i16 = if input.button_pressed { 1 } else { -1 };
    let speed = I3F13::from_fixed(p.absolute_speed) * direction;
    let position_delta = speed.wide_mul(I16F0::from_bits(input.ms as i16 - s.ts as i16)) / 1000;
    let new_position = s.position + position_delta.to_fixed::<I3F13>();
    let new_position = I3F13::min(p.max_position, I3F13::max(p.min_position, new_position));
    let new_state = State {
        position: new_position,
        ts: input.ms,
    };
    let outputs = Outputs {
        position: new_position,
        button: input.button_pressed,
    };
    (new_state, outputs)
}

pub struct InputPeripherals<T: Chrono, IP: InputPin> {
    pub chrono: T,
    pub button: IP,
}

fn read_inputs<T: Chrono, IP: InputPin>(ps: &mut InputPeripherals<T, IP>) -> Inputs {
    Inputs {
        ms: ps.chrono.millis(),
        button_pressed: ps.button.is_low().unwrap(),
    }
}

pub struct OutputPeripherals<S: embedded_hal::pwm::SetDutyCycle, SERIAL: ufmt::uWrite> {
    pub servo: S,
    pub serial: SERIAL,
}

fn apply_outputs<S: SetDutyCycle, SERIAL: ufmt::uWrite>(
    p: &Parameters,
    outputs: Outputs,
    op: &mut OutputPeripherals<S, SERIAL>,
) -> () {
    let max_duty = op.servo.max_duty_cycle();

    let position_range = p.max_position - p.min_position;

    let duty_range_us = p.max_position_us - p.min_position_us;

    let t = (outputs.position - p.min_position).to_bits();

    let duty_us = p.min_position_us
        + (((t as u32) * (duty_range_us as u32)) / (position_range.to_bits() as u32)) as u16;

    let duty: u16 = ((duty_us as u32) * (max_duty as u32) / (p.pwm_period_us as u32)) as u16;

    ufmt::uwrite!(
        &mut op.serial,
        "button={}\tmax_duty={}\tduty={}\tposition_range={}\tduty_range_us={}\tt={}\tduty_us={}\n",
        outputs.button,
        max_duty,
        duty,
        position_range.to_bits(),
        duty_range_us,
        t,
        duty_us,
    )
    .unwrap_or_default();
    op.servo.set_duty_cycle(duty).unwrap_or(())
}
