use embedded_hal::digital::{InputPin, OutputPin, PinState};
use rand::{rngs::SmallRng, RngCore};

use crate::chrono::Chrono;

pub fn run<C: Chrono, IP: InputPin, OP: OutputPin>(
    params: &Parameters,
    ips: &mut InputPeripherals<C, IP>,
    ops: &mut OutputPeripherals<OP>,
) -> ! {
    let mut state = State::ShowingResult(DiceRoll::from_int_modulo(6));
    loop {
        let inputs = read_inputs(ips);
        // Generate a new state of the game instead of mutating the current state
        let new_state = update_state(params, state, inputs);
        state = new_state;
        // Build the appropriate output values from the new state
        let outputs = make_outputs(&state);
        apply_outputs(outputs, ops)
    }
}

// A wrapper that contains a dice roll between 1 and 6
struct DiceRoll(u8);

impl DiceRoll {
    fn from_int_modulo(v: u32) -> DiceRoll {
        DiceRoll((((v - 1) % 6) + 1) as u8)
    }

    fn to_u8(&self) -> u8 {
        self.0
    }
}

struct Inputs {
    ms: u32,
    button_pressed: bool,
    roll: DiceRoll,
}

struct Outputs {
    leds: [bool; 5],
}

enum State {
    Blinking(
        u8, /*blink index */
        DiceRoll,
        u32, /* blink end time */
    ),
    ShowingResult(DiceRoll),
}

pub struct Parameters {
    // The number of blink before the players have to press their button
    pub nb_blinks: u8,
    pub first_blink_duration_ms: u32,
    pub last_blink_duration_ms: u32,
}

fn update_state(p: &Parameters, s: State, input: Inputs) -> State {
    match s {
        State::Blinking(blink_number, _, blink_end_time) => {
            if input.ms > blink_end_time {
                // If we spent enough time on the current dice roll, move to the
                // next state
                let next_blink_number = blink_number + 1;
                if next_blink_number > p.nb_blinks {
                    // If it has blinked enough times, show the result
                    State::ShowingResult(input.roll)
                } else {
                    // If it has not blinked, blink on another dice roll, for a
                    // longer duration
                    State::Blinking(
                        next_blink_number,
                        input.roll,
                        input.ms + blink_duration_at(p, next_blink_number),
                    )
                }
            } else {
                // Nothing to change, keep the same intermediate state
                s
            }
        }
        State::ShowingResult(_) => {
            if input.button_pressed {
                // If the button is pressed, throw the dice again
                if p.nb_blinks > 0 {
                    // If we configured some blinking, blink
                    State::Blinking(0, input.roll, p.first_blink_duration_ms + input.ms)
                } else {
                    // Otherwise, move directly to a new result
                    State::ShowingResult(input.roll)
                }
            } else {
                // If the button was not pressed, do nothing
                s
            }
        }
    }
}

// computes the duration of the i-th blink, using an affine function
fn blink_duration_at(p: &Parameters, i: u8) -> u32 {
    p.first_blink_duration_ms
        + ((p.last_blink_duration_ms - p.first_blink_duration_ms) * i as u32) / p.nb_blinks as u32
}

fn make_outputs(s: &State) -> Outputs {
    let roll = match s {
        State::Blinking(_, roll, _) => roll,
        State::ShowingResult(roll) => roll,
    };
    Outputs {
        leds: u8_to_array5(roll.to_u8()),
    }
}

// Build a dice led pattern from a u8
fn u8_to_array5(v: u8) -> [bool; 5] {
    // r[0]    r[1]
    //     r[2]
    // r[3]    r[4]
    match v {
        // 0 0
        //  1
        // 0 0
        1 => [false, false, true, false, false],
        // 1 0
        //  0
        // 0 1
        2 => [true, false, false, false, true],
        // 1 0
        //  1
        // 0 1
        3 => [true, false, true, false, true],
        // 1 1
        //  0
        // 1 1
        4 => [true, true, false, true, true],
        // 1 1
        //  1
        // 1 1
        5 => [true, true, true, true, true],
        _ => [false, false, false, false, false],
    }
}

pub struct InputPeripherals<T: Chrono, IP: InputPin> {
    pub chrono: T,
    pub button: IP,
    // The rng is setup as an input peripheral to keep the core logic pure
    pub rng: SmallRng,
}

fn read_inputs<T: Chrono, IP: InputPin>(ps: &mut InputPeripherals<T, IP>) -> Inputs {
    Inputs {
        ms: ps.chrono.millis(),
        button_pressed: ps.button.is_low().unwrap(),
        // We get a new roll on each loop, even if we don't use it everytime
        roll: DiceRoll::from_int_modulo(ps.rng.next_u32()),
    }
}

pub struct OutputPeripherals<OP: OutputPin> {
    pub leds: [OP; 5],
}

fn apply_outputs<T: OutputPin>(outputs: Outputs, pins: &mut OutputPeripherals<T>) -> () {
    // no alloc zip magic
    for (pin, state) in pins.leds.iter_mut().zip(outputs.leds.iter()) {
        pin.set_state(PinState::from(*state)).unwrap()
    }
}
