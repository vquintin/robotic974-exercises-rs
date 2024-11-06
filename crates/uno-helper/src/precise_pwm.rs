use arduino_hal::pac::TC1;
use embedded_hal::pwm::ErrorKind;
use uno_lib::timer_1::PreciseTimerParams;

#[inline]
pub fn new_precise_pwm(t: TC1, ptp: PreciseTimerParams) -> PreciseTimer1 {
    t.icr1.write(|w| w.bits(ptp.icr1));
    t.tccr1a.write(|w| w.wgm1().bits(0b00).com1a().bits(0b10));
    t.tccr1b.write(|w| {
        let w = w.wgm1().bits(0b10).cs1();
        match ptp.prescaler {
            8 => w.prescale_8(),
            64 => w.prescale_64(),
            256 => w.prescale_256(),
            1024 => w.prescale_1024(),
            _ => panic!(),
        }
    });
    PreciseTimer1(t)
}

pub struct PreciseTimer1(TC1);

impl embedded_hal::pwm::SetDutyCycle for PreciseTimer1 {
    fn max_duty_cycle(&self) -> u16 {
        self.0.icr1.read().bits()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        Ok(self.0.ocr1a.write(|w| w.bits(duty)))
    }
}

impl embedded_hal::pwm::ErrorType for PreciseTimer1 {
    type Error = PwmError;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PwmError {
    DutyCycleTooLarge,
}

impl embedded_hal::pwm::Error for PwmError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}
