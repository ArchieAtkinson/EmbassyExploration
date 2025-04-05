#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]

pub mod cli;
mod frame_ascii;
pub mod matrix;
pub mod scroller;
pub mod transport;
pub mod uarte;

#[cfg(test)]
pub mod gpio_mock;

pub mod prelude {
    #[cfg(not(test))]
    pub use defmt::{debug, error, info, warn};

    #[cfg(test)]
    pub use log::{debug, error, info, warn};
}

#[cfg(test)]
#[ctor::ctor]
fn ctor() {
    pretty_env_logger::init();
}
