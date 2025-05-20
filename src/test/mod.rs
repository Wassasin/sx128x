use core::convert::Infallible;

use embedded_hal::digital::{ErrorType, OutputPin};
use embedded_hal_async::{delay::DelayNs, digital::Wait};
use embedded_hal_mock::eh1::spi::Transaction;

mod hl;
mod ll;

fn cmd_g(cmd: u8, in_array: &[u8], out_array: &[u8]) -> Vec<Transaction<u8>> {
    vec![
        Transaction::transaction_start(),
        Transaction::write(cmd),
        Transaction::write_vec(in_array.to_vec()),
        Transaction::read_vec(out_array.to_vec()),
        Transaction::transaction_end(),
    ]
}

fn cmd_w(cmd: u8, in_array: &[u8]) -> Vec<Transaction<u8>> {
    cmd_g(cmd, in_array, &[])
}

fn cmd_r(cmd: u8, out_array: &[u8]) -> Vec<Transaction<u8>> {
    cmd_g(cmd, &[0x00], out_array)
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

fn reg_w(reg: u16, in_array: &[u8]) -> Vec<Transaction<u8>> {
    vec![
        Transaction::transaction_start(),
        Transaction::write_vec([[0x18].as_slice(), &reg.to_be_bytes()].concat().to_vec()),
        Transaction::write_vec(in_array.to_vec()),
        Transaction::transaction_end(),
    ]
}

fn buf_r(offset: u8, out_array: &[u8]) -> Vec<Transaction<u8>> {
    vec![
        Transaction::transaction_start(),
        Transaction::write_vec([0x1B, offset, 0x00].to_vec()),
        Transaction::read_vec(out_array.to_vec()),
        Transaction::transaction_end(),
    ]
}

fn buf_w(offset: u8, in_array: &[u8]) -> Vec<Transaction<u8>> {
    vec![
        Transaction::transaction_start(),
        Transaction::write_vec([0x1A, offset].to_vec()),
        Transaction::write_vec(in_array.to_vec()),
        Transaction::transaction_end(),
    ]
}

struct MockWait;

impl ErrorType for MockWait {
    type Error = Infallible;
}

impl Wait for MockWait {
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

impl ErrorType for MockOutput {
    type Error = Infallible;
}

struct MockOutput;

impl OutputPin for MockOutput {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct MockDelay;

impl DelayNs for MockDelay {
    async fn delay_ns(&mut self, _ns: u32) {}
}
