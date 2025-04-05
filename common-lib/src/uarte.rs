use embassy_nrf::timer::{self};
use embassy_nrf::uarte;
use thiserror::Error;

mod nrf {
    pub use embassy_nrf::uarte::UarteRxWithIdle;
    pub use embassy_nrf::uarte::UarteTx;
}

#[derive(Error, Debug)]
pub enum UarteTxError {
    #[error("")]
    Error,
}

#[cfg_attr(test, mockall::automock)]
pub trait UarteTx {
    async fn write(&mut self, buffer: &[u8]) -> Result<(), UarteTxError>;
    async fn write_from_ram(&mut self, buffer: &[u8]) -> Result<(), UarteTxError>;
}

impl<'d, T: uarte::Instance> UarteTx for nrf::UarteTx<'d, T> {
    async fn write(&mut self, buffer: &[u8]) -> Result<(), UarteTxError> {
        match self.write(buffer).await {
            Ok(()) => Ok(()),
            Err(_) => Err(UarteTxError::Error),
        }
    }
    async fn write_from_ram(&mut self, buffer: &[u8]) -> Result<(), UarteTxError> {
        match self.write_from_ram(buffer).await {
            Ok(()) => Ok(()),
            Err(_) => Err(UarteTxError::Error),
        }
    }
}

#[derive(Error, Debug)]
pub enum UarteRxError {
    #[error("")]
    Error,
}

#[cfg_attr(test, mockall::automock)]
pub trait UarteRx {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<(), UarteRxError>;
    async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, UarteRxError>;
}

impl<'d, T: uarte::Instance, U: timer::Instance> UarteRx for nrf::UarteRxWithIdle<'d, T, U> {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<(), UarteRxError> {
        match self.read(buffer).await {
            Ok(()) => Ok(()),
            Err(_) => Err(UarteRxError::Error),
        }
    }
    async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, UarteRxError> {
        match self.read_until_idle(buffer).await {
            Ok(size) => Ok(size),
            Err(_) => Err(UarteRxError::Error),
        }
    }
}
