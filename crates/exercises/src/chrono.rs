//! This module contains a trait to get the millis elapsed since the last reset
//! call.

pub trait Chrono {
    fn millis(&self) -> u32;
    fn reset(&self);
}
