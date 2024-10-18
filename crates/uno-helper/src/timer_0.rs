//! This module is copied from Rahix example
//! https://github.com/Rahix/avr-hal/blob/main/examples/arduino-uno/src/bin/uno-millis.rs
//! It uses the timer 0 for the millis, as in the standard arduino lib
//! Although it uses a different principle (and interrupt)
//! See https://blog.rahix.de/005-avr-hal-millis/ for details
//!
//! You need to enable interrupts in order for the timer to work

use core::cell;
use exercises::chrono::Chrono;
use panic_halt as _;

// Possible Values:
//
// ╔═══════════╦══════════════╦═══════════════════╗
// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
// ╠═══════════╬══════════════╬═══════════════════╣
// ║        64 ║          250 ║              1 ms ║
// ║       256 ║          125 ║              2 ms ║
// ║       256 ║          250 ║              4 ms ║
// ║      1024 ║          125 ║              8 ms ║
// ║      1024 ║          250 ║             16 ms ║
// ╚═══════════╩══════════════╩═══════════════════╝
const PRESCALER: u32 = 1024;
const TIMER_COUNTS: u32 = 125;

const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

static MILLIS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u32>> =
    avr_device::interrupt::Mutex::new(cell::Cell::new(0));

pub struct Chrono0 {
    tc0: arduino_hal::pac::TC0,
}

impl Chrono0 {
    pub fn new(tc0: arduino_hal::pac::TC0) -> Chrono0 {
        let c = Chrono0 { tc0 };
        c.reset();
        c
    }
}

impl exercises::chrono::Chrono for Chrono0 {
    fn millis(&self) -> u32 {
        avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
    }

    fn reset(&self) {
        // Configure the timer for the above interval (in CTC mode)
        // and enable its interrupt.
        self.tc0.tccr0a.write(|w| w.wgm0().ctc());
        self.tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS as u8));
        self.tc0.tccr0b.write(|w| match PRESCALER {
            8 => w.cs0().prescale_8(),
            64 => w.cs0().prescale_64(),
            256 => w.cs0().prescale_256(),
            1024 => w.cs0().prescale_1024(),
            _ => panic!(),
        });
        self.tc0.timsk0.write(|w| w.ocie0a().set_bit());

        // Reset the global millisecond counter
        avr_device::interrupt::free(|cs| {
            MILLIS_COUNTER.borrow(cs).set(0);
        });
    }
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MILLIS_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + MILLIS_INCREMENT);
    })
}
