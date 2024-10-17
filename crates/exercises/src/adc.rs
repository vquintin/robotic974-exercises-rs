use fixed::types::U1F15;

pub trait Adc<PIN> {
    fn read_nonblocking(&mut self, pin: &PIN) -> nb::Result<U1F15, core::convert::Infallible>;
}
