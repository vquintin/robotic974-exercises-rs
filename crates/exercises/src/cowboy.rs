use embedded_hal::digital::{InputPin, OutputPin, PinState};

use crate::chrono::Chrono;

pub fn run<C: Chrono, IP: InputPin, OP: OutputPin>(
    params: &Parameters,
    ips: &mut InputPeripherals<C, IP>,
    ops: &mut OutputPeripherals<OP>,
) -> ! {
    let mut state = State::Blinking(0);
    loop {
        let inputs = read_inputs(ips);
        // Generate a new state of the game instead of mutating the current state
        let (new_state, outputs) = advance(params, state, inputs);
        state = new_state;
        apply_outputs(outputs, ops)
    }
}

struct Inputs {
    ms: u32,
    // true when the buttons are pressed
    left_button: bool,
    right_button: bool,
}

struct Outputs {
    left_led: bool,
    right_led: bool,
}

pub enum State {
    // Blinking is the pregame wait, when the leds are flashing to make the players ready
    // Pressing the button in this state makes the player loses
    // It contains the time at which it started blinking to know when to start the game.
    Blinking(u32),
    // WaitingForFastest means the mcu is waiting for at least one of the players to
    // press their button
    WaitingForFastest(),
    // ShowingWinner is the endgame state, when the mcu shows the winner by turning
    // one or two leds (on ex-aequo)
    // It contains the time at which the mcu started to show the winner in order to
    // know when to restart the game.
    ShowingWinner(u32, bool, bool),
}

pub struct Parameters {
    // The number of blink before the players have to press their button
    pub nb_blinks: u32,
    // The whole duration of the pre-game blinking
    pub blink_duration_ms: u32,
    // The duration of the game end results
    pub show_winner_duration: u32,
}

fn advance(p: &Parameters, s: State, input: Inputs) -> (State, Outputs) {
    let led_off = Outputs {
        left_led: false,
        right_led: false,
    };
    return match s {
        State::Blinking(start) => {
            if start > input.ms {
                // nitpicky check: detect clock rollover if the mcu is on for a
                // long time
                (State::Blinking(input.ms), led_off)
            } else if input.ms > p.blink_duration_ms + start {
                // If the blinking period has ended, wait the fastest player with
                // the leds turned off
                (State::WaitingForFastest(), led_off)
            } else if input.left_button || input.right_button {
                // If at least one the button was pressed (can be both), the players
                // who pressed have lost.
                // Notice the left/right inversion to make the other win.
                (
                    State::ShowingWinner(input.ms, input.right_button, input.left_button),
                    led_off,
                )
            } else {
                // Make the leds blink p.nb_blinks time
                let since_blink = input.ms - start;
                let period = p.blink_duration_ms / p.nb_blinks;
                let cycle_position = since_blink % period;
                let led_on = cycle_position > period / 2;
                (
                    s,
                    Outputs {
                        left_led: led_on,
                        right_led: led_on,
                    },
                )
            }
        }
        State::WaitingForFastest() => {
            // If a player or both have pressed during the game, move to showing the results.
            // If nothing happened, keep waiting
            let new_state = if input.left_button || input.right_button {
                State::ShowingWinner(input.ms, input.left_button, input.right_button)
            } else {
                s
            };
            (new_state, led_off)
        }
        State::ShowingWinner(start, left_won, right_won) => {
            let result = Outputs {
                left_led: left_won,
                right_led: right_won,
            };
            if start > input.ms {
                // nitpicky check: detect clock rollover if the mcu is on for a
                // long time
                (State::ShowingWinner(input.ms, left_won, right_won), result)
            } else if input.ms > p.show_winner_duration + start {
                // If the mcu showed the result for long enough, restart the game
                (State::Blinking(input.ms), led_off)
            } else {
                // If we're still showing the results, show them
                (s, result)
            }
        }
    };
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

pub struct OutputPeripherals<OP: OutputPin> {
    pub left_led: OP,
    pub right_led: OP,
}

fn apply_outputs<T: OutputPin>(outputs: Outputs, pins: &mut OutputPeripherals<T>) -> () {
    // Set the state of the two leds
    pins.left_led
        .set_state(PinState::from(outputs.left_led))
        .unwrap();
    pins.right_led
        .set_state(PinState::from(outputs.right_led))
        .unwrap();
}
