use arduino_hal::pac::tc1::{icr1::ICR1_SPEC, ocr1a::OCR1A_SPEC, ocr1b::OCR1B_SPEC, RegisterBlock};
use embedded_hal::pwm::ErrorKind;
use uno_lib::timer_1::PreciseTimerParams;

#[inline]
pub fn new_precise_pwm<'a>(
    x: &'a RegisterBlock,
    ptp: PreciseTimerParams,
) -> (PreciseTimer1A<'a>, PreciseTimer1B<'a>) {
    //let x = t.deref();
    let arduino_hal::pac::tc1::RegisterBlock {
        icr1,
        tccr1a,
        tccr1b,
        ocr1a,
        ocr1b,
        ..
    } = x;

    icr1.write(|w| w.bits(ptp.icr1));
    tccr1a.write(|w| w.wgm1().bits(0b00).com1a().bits(0b10));
    tccr1b.write(|w| {
        let w = w.wgm1().bits(0b10).cs1();
        match ptp.prescaler {
            8 => w.prescale_8(),
            64 => w.prescale_64(),
            256 => w.prescale_256(),
            1024 => w.prescale_1024(),
            _ => panic!(),
        }
    });
    (
        PreciseTimer1A { icr1, ocr1a },
        PreciseTimer1B { icr1, ocr1b },
    )
}

pub struct PreciseTimer1A<'a> {
    icr1: &'a avr_device::generic::Reg<ICR1_SPEC>,
    ocr1a: &'a avr_device::generic::Reg<OCR1A_SPEC>,
}

impl<'a> embedded_hal::pwm::SetDutyCycle for PreciseTimer1A<'a> {
    fn max_duty_cycle(&self) -> u16 {
        self.icr1.read().bits()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        Ok(self.ocr1a.write(|w| w.bits(duty)))
    }
}

impl<'a> embedded_hal::pwm::ErrorType for PreciseTimer1A<'a> {
    type Error = PwmError;
}

pub struct PreciseTimer1B<'a> {
    icr1: &'a avr_device::generic::Reg<ICR1_SPEC>,
    ocr1b: &'a avr_device::generic::Reg<OCR1B_SPEC>,
}

impl<'a> embedded_hal::pwm::SetDutyCycle for PreciseTimer1B<'a> {
    fn max_duty_cycle(&self) -> u16 {
        self.icr1.read().bits()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        Ok(self.ocr1b.write(|w| w.bits(duty)))
    }
}

impl<'a> embedded_hal::pwm::ErrorType for PreciseTimer1B<'a> {
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
