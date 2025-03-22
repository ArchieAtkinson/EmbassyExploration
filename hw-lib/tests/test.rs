#![no_std]
#![no_main]

use defmt_rtt as _; // Global logger
use panic_probe as _;

use embassy_nrf::Peripherals;

struct State {
    pub peripherals: Peripherals,
}

#[defmt_test::tests]
mod tests {
    use super::State;
    use defmt::assert;
    use embassy_nrf::config::Config;

    #[init]
    fn init() -> State {
        State {
            peripherals: embassy_nrf::init(Config::default()),
        }
    }

    #[test]
    fn it_works(_state: &mut State) {
        assert!(true);
    }
}
