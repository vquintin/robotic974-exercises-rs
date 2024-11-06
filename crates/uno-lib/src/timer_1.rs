use fixed::types::U24F8;

const POSSIBLE_PRESCALERS: [u16; 5] = [1, 8, 64, 256, 1024];

// This fonction is executed at compile time when possible
// It computes the prescaler and icr1 given:
// - a CPU clock
// - a minimum frequency in Hz
// - a frequency range in Hz
// The returned option is empty if no settings is found in
// [min_frequency; min_frequncy + frequency_range]
pub const fn compute_timer_params(
    clock: U24F8,
    min_frequency: U24F8,
    frequncy_range: U24F8,
) -> Option<PreciseTimerParams> {
    let mut i = 0;
    while i < POSSIBLE_PRESCALERS.len() {
        let ps = POSSIBLE_PRESCALERS[i] as u32;
        let n = clock.to_bits() / (2 * min_frequency.to_bits() * ps);
        if n > ((1 << 16) - 1) {
            i += 1;
            continue;
        }
        let remaining_cpu_cycles = clock.to_bits() % (2 * n * ps);
        let limit_remaining = frequncy_range.to_bits() * (2 * n * ps);
        if remaining_cpu_cycles > limit_remaining {
            return None;
        }
        return Some(PreciseTimerParams {
            prescaler: POSSIBLE_PRESCALERS[i],
            icr1: n as u16,
        });
    }
    None
}

#[derive(Debug, PartialEq)]
pub struct PreciseTimerParams {
    pub icr1: u16,
    pub prescaler: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outputs() {
        struct TestCase {
            f: U24F8,
            d: U24F8,
            expected: Option<PreciseTimerParams>,
        }
        let cases = [
            TestCase {
                f: U24F8::lit("50.0"),
                d: U24F8::lit("0.0"),
                expected: Some(PreciseTimerParams {
                    icr1: 20_000,
                    prescaler: 8,
                }),
            },
            TestCase {
                f: U24F8::lit("60.0"),
                d: U24F8::lit("0.0"),
                expected: None,
            },
            TestCase {
                f: U24F8::lit("60.0"),
                d: U24F8::lit("0.01"),
                expected: Some(PreciseTimerParams {
                    icr1: 16666,
                    prescaler: 8,
                }),
            },
            TestCase {
                f: U24F8::lit("16_000"),
                d: U24F8::lit("0"),
                expected: Some(PreciseTimerParams {
                    icr1: 500,
                    prescaler: 1,
                }),
            },
            TestCase {
                f: U24F8::lit("0.125"),
                d: U24F8::lit("0"),
                expected: Some(PreciseTimerParams {
                    icr1: 62500,
                    prescaler: 1024,
                }),
            },
            TestCase {
                f: U24F8::lit("125"),
                d: U24F8::lit("0"),
                expected: Some(PreciseTimerParams {
                    icr1: 64000,
                    prescaler: 1,
                }),
            },
        ];
        for tc in cases {
            let actual = compute_timer_params(U24F8::lit("16_000_000"), tc.f, tc.d);
            assert_eq!(tc.expected, actual, "wrong params")
        }
    }
}
