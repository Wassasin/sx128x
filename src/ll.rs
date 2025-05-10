use core::convert::Infallible;

use device_driver::{AsyncCommandInterface, AsyncRegisterInterface};
use embedded_hal::spi::Operation;
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

pub struct Interface<SPI, BUSY> {
    spi: SPI,
    busy: BUSY,
}

impl<SPI, BUSY> AsyncCommandInterface for Interface<SPI, BUSY>
where
    SPI: SpiDevice,
    BUSY: Wait<Error = Infallible>,
{
    type Error = SPI::Error;
    type AddressType = u8;

    async fn dispatch_command(
        &mut self,
        address: Self::AddressType,
        _size_bits_in: u32,
        input: &[u8],
        _size_bits_out: u32,
        output: &mut [u8],
    ) -> Result<(), Self::Error> {
        let command = [address];
        let mut operations = [
            Operation::Write(&command),
            Operation::Write(input),
            Operation::Read(output),
        ];

        let _ = self.busy.wait_for_low().await;
        self.spi.transaction(&mut operations).await?;
        let _ = self.busy.wait_for_low().await;
        Ok(())
    }
}

impl<SPI, BUSY> AsyncRegisterInterface for Interface<SPI, BUSY>
where
    SPI: SpiDevice,
    BUSY: Wait<Error = Infallible>,
{
    type Error = SPI::Error;
    type AddressType = u16;

    async fn write_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let address = address.to_be_bytes();
        let command = [0x18, address[0], address[1]];

        let mut operations = [Operation::Write(&command), Operation::Write(data)];

        let _ = self.busy.wait_for_low().await;
        self.spi.transaction(&mut operations).await?;
        let _ = self.busy.wait_for_low().await;
        Ok(())
    }

    async fn read_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        let address = address.to_be_bytes();
        let command = [0x19, address[0], address[1], 0x00];

        let mut operations = [Operation::Write(&command), Operation::Read(data)];

        let _ = self.busy.wait_for_low().await;
        self.spi.transaction(&mut operations).await?;
        let _ = self.busy.wait_for_low().await;
        Ok(())
    }
}

impl<SPI, BUSY> Interface<SPI, BUSY> {
    pub fn new(spi: SPI, busy: BUSY) -> Self {
        Self { spi, busy }
    }

    pub fn take(self) -> (SPI, BUSY) {
        (self.spi, self.busy)
    }
}

device_driver::create_device!(
    device_name: Device,
    manifest: "device.yaml"
);
