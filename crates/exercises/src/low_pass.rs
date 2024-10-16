use fixed::{
    traits::{Fixed, ToFixed},
    types::U1F15,
};

// FixedLowPassState is exponential moving average implemented using fixed point
// arithmetic.
// It is only able to represent values x such that 0 <= x < 2
// It is useful to debounce a button, or filter noise out of an adc
pub struct FixedLowPassState {
    // The current value of the average
    current_average: U1F15,
    // The current (latest) time at which the average was computed
    current_ts: u32,
}

// FixedLowPassParams are the const params of the low pass
// They're in a separate struct so they can stay in the flash and not consume RAM.
pub struct FixedLowPassParams {
    // A value between 0 and 1 (U1F15::ONE) to decide of the average window size
    // A bigger lambda makes an average over a longer time (more noise filtering)
    // A lower lamdba makes the average move more quickly
    // The window duration is approx 1 / lambda, in ms
    pub lambda: U1F15,
}

impl FixedLowPassState {
    pub fn new() -> FixedLowPassState {
        FixedLowPassState {
            current_ts: 0,
            current_average: U1F15::ZERO,
        }
    }

    pub fn advance(self, p: &FixedLowPassParams, x: U1F15, timestamp: u32) -> FixedLowPassState {
        let delta_t = timestamp - self.current_ts;
        let k = pow_int(U1F15::ONE - p.lambda, delta_t, U1F15::ONE);
        let new_average = k.wide_mul(self.current_average) + (U1F15::ONE - k).wide_mul(x);
        FixedLowPassState {
            current_average: new_average.to_fixed(),
            current_ts: timestamp,
        }
    }

    pub fn current_val(&self) -> U1F15 {
        self.current_average
    }
}

// computes a fixed point integer exponentiation
fn pow_int<F: Fixed>(a: F, b: u32, one: F) -> F {
    let mut p = a;
    let mut q = b;
    let mut acc = one;
    while q != 0 {
        if q & 0x1 == 1 {
            acc = acc * p
        }
        // Avoid p overflow on last step
        if q > 1 {
            p = p * p;
        }
        q = q >> 1;
    }
    acc
}

#[cfg(test)]
mod tests {
    use fixed::types::U6F26;

    use super::*;

    #[test]
    fn test_outputs() {
        struct TestCase {
            a: U6F26,
            b: u32,
            expected: U6F26,
        }
        let cases = [
            TestCase {
                a: U6F26::unwrapped_from_num(2),
                b: 0,
                expected: U6F26::ONE,
            },
            TestCase {
                a: U6F26::unwrapped_from_num(2),
                b: 1,
                expected: U6F26::unwrapped_from_num(2),
            },
            TestCase {
                a: U6F26::unwrapped_from_num(2),
                b: 5,
                expected: U6F26::unwrapped_from_num(32),
            },
            TestCase {
                a: U6F26::unwrapped_from_num(0.5),
                b: 2,
                expected: U6F26::unwrapped_from_num(0.25),
            },
        ];
        for tc in cases {
            let actual = pow_int(tc.a, tc.b, U6F26::ONE);
            assert_eq!(tc.expected, actual, "wrong exponentiation")
        }
    }
}
