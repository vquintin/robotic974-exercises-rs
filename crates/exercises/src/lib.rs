#![no_std]

pub mod adc;
pub mod chenillard;
pub mod chrono;
pub mod cowboy;
pub mod debounce;
pub mod low_pass;
pub mod vumetre;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
