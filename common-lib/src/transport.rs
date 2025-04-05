use core::str;
use heapless::String;
use thiserror::Error;

use crate::uarte::{UarteRx, UarteRxError};

pub trait Transport {
    type Error;
    async fn next_line(&mut self) -> Result<Option<String<MAX_LINE_LEN>>, Self::Error>;
}

pub static MAX_LINE_LEN: usize = 80;

struct UartTransport<T: UarteRx> {
    buf: String<256>,
    rx: T,
}

impl<T: UarteRx> UartTransport<T> {
    pub fn new(uarte_rx: T) -> Self {
        Self {
            buf: String::new(),
            rx: uarte_rx,
        }
    }
}

impl<T: UarteRx> Transport for UartTransport<T> {
    type Error = UarteRxError;
    async fn next_line(&mut self) -> Result<Option<String<MAX_LINE_LEN>>, Self::Error> {
        let mut input_buffer = [0; 64];

        let size = self.rx.read_until_idle(&mut input_buffer).await?;

        let input_string = str::from_utf8(&input_buffer[0..size]).unwrap();

        self.buf.push_str(input_string).unwrap();

        let newline_loc = self.buf.find("\n");
        if let Some(loc) = newline_loc {
            let mut output = String::new();
            output.push_str(&self.buf.as_str()[0..loc + 1]).unwrap();

            let mut tmp = String::new();
            let buf_size = self.buf.len();
            tmp.push_str(&self.buf.as_str()[loc + 1..buf_size]).unwrap();
            self.buf = tmp;
            return Ok(Some(output));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use crate::uarte::MockUarteRx;

    use super::*;

    #[futures_test::test]
    async fn test_next_line_with_provided_line() {
        let mut mock = MockUarteRx::new();

        let test_string = "Test\n";
        mock.expect_read_until_idle().returning(|buf| {
            buf[..test_string.len()].copy_from_slice(test_string.as_bytes());
            Ok(test_string.len())
        });

        let mut transport = UartTransport::new(mock);
        let output_string = transport.next_line().await.unwrap().unwrap();

        assert_eq!(test_string, output_string.as_str());
    }

    #[futures_test::test]
    async fn test_next_line_with_split_line() {
        let mut mock = MockUarteRx::new();

        let test_string_1 = "Tes";
        let test_string_2 = "t\n";
        mock.expect_read_until_idle().times(1).returning(|buf| {
            buf[..test_string_1.len()].copy_from_slice(test_string_1.as_bytes());
            Ok(test_string_1.len())
        });

        mock.expect_read_until_idle().times(1).returning(|buf| {
            buf[..test_string_2.len()].copy_from_slice(test_string_2.as_bytes());
            Ok(test_string_2.len())
        });

        let mut transport = UartTransport::new(mock);

        assert!(transport.next_line().await.unwrap().is_none());

        let output_string = transport.next_line().await.unwrap().unwrap();
        let expected_string = test_string_1.to_owned() + test_string_2;
        assert_eq!(expected_string, output_string.as_str());
    }

    #[futures_test::test]
    async fn test_next_line_with_two_lines() {
        let mut mock = MockUarteRx::new();

        let test_string = "Test\nString\n";
        mock.expect_read_until_idle().times(1).returning(|buf| {
            buf[..test_string.len()].copy_from_slice(test_string.as_bytes());
            Ok(test_string.len())
        });

        mock.expect_read_until_idle().times(1).returning(|_| Ok(0));

        let mut transport = UartTransport::new(mock);

        let output_string = transport.next_line().await.unwrap().unwrap();
        let expected_string = "Test\n";
        assert_eq!(expected_string, output_string.as_str());

        let output_string = transport.next_line().await.unwrap().unwrap();
        let expected_string = "String\n";
        assert_eq!(expected_string, output_string.as_str());
    }
}
