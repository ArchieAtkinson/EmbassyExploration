use core::convert::Infallible;

use embedded_hal::digital::{ErrorType, OutputPin, PinState, StatefulOutputPin};
use mockall::*;

mock! {
    pub Output {}

    impl ErrorType for Output {
        type Error = Infallible;
    }

    impl OutputPin for Output {
        fn set_high(&mut self) -> Result<(), <Self as ErrorType>::Error> { todo!() }
        fn set_low(&mut self) -> Result<(), <Self as ErrorType>::Error> { todo!() }
    }

    impl StatefulOutputPin for Output {
        fn is_set_high(&mut self) -> Result<bool, <Self as ErrorType>::Error> { todo!() }
        fn is_set_low(&mut self) -> Result<bool, <Self as ErrorType>::Error> { todo!() }
    }
}

pub fn expect_gpio(pin: &mut MockOutput, state: PinState) {
    let mut seq = Sequence::new();

    match state {
        PinState::High => {
            let _ = pin
                .expect_set_high()
                .once()
                .returning(|| Ok(()))
                .in_sequence(&mut seq);
        }
        PinState::Low => {
            let _ = pin
                .expect_set_low()
                .once()
                .returning(|| Ok(()))
                .in_sequence(&mut seq);
        }
    }
}
