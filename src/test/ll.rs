use core::convert::Infallible;

use embedded_hal::digital::ErrorType;
use embedded_hal_async::digital::Wait;
use embedded_hal_mock::eh1::spi::{Mock, Transaction};

use crate::ll;

fn cmd(cmd: u8, in_array: &[u8], out_array: &[u8]) -> Vec<Transaction<u8>> {
    vec![
        Transaction::transaction_start(),
        Transaction::write(cmd),
        Transaction::write_vec(in_array.to_vec()),
        Transaction::read_vec(out_array.to_vec()),
        Transaction::transaction_end(),
    ]
}

fn reg_r(reg: u16, out_array: &[u8]) -> Vec<Transaction<u8>> {
    vec![
        Transaction::transaction_start(),
        Transaction::write_vec(
            [[0x19].as_slice(), &reg.to_be_bytes(), &[0x00]]
                .concat()
                .to_vec(),
        ),
        Transaction::read_vec(out_array.to_vec()),
        Transaction::transaction_end(),
    ]
}

struct MockBusy;

impl ErrorType for MockBusy {
    type Error = Infallible;
}

impl Wait for MockBusy {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn command() {
    let expectations = [cmd(0x84, &[0b10], &[])];
    let mut spi = Mock::new(expectations.iter().flatten());
    let mut ll = ll::Device::new(ll::Interface::new(&mut spi, MockBusy));

    embassy_futures::block_on(async {
        ll.set_sleep()
            .dispatch_async(|cmd| cmd.set_buffer_retention(true))
            .await
            .unwrap();
    });

    spi.done();
}

#[test]
fn register() {
    let expectations = [reg_r(0x153, &[0xA9, 0xB5])];
    let mut spi = Mock::new(expectations.iter().flatten());
    let mut ll = ll::Device::new(ll::Interface::new(&mut spi, MockBusy));

    embassy_futures::block_on(async {
        let fw = ll.firmware_versions().read_async().await.unwrap().value();
        assert_eq!(fw, ll::FirmwareVersion::Version1);
    });

    spi.done();
}
