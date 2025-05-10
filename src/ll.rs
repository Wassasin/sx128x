use core::convert::Infallible;

use device_driver::{
    AsyncBufferInterface, AsyncCommandInterface, AsyncRegisterInterface, BufferInterfaceError,
};
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

impl<SPI, BUSY> BufferInterfaceError for Interface<SPI, BUSY>
where
    SPI: SpiDevice,
{
    type Error = SPI::Error;
}

impl<SPI, BUSY> AsyncBufferInterface for Interface<SPI, BUSY>
where
    SPI: SpiDevice,
    BUSY: Wait<Error = Infallible>,
{
    type AddressType = u8;

    async fn write(
        &mut self,
        address: Self::AddressType,
        buf: &[u8],
    ) -> Result<usize, Self::Error> {
        let command = [0x1A, address];
        let mut operations = [Operation::Write(&command), Operation::Write(buf)];

        let _ = self.busy.wait_for_low().await;
        self.spi.transaction(&mut operations).await?;
        let _ = self.busy.wait_for_low().await;

        Ok(buf.len())
    }

    async fn flush(&mut self, _address: Self::AddressType) -> Result<(), Self::Error> {
        // Do nothing
        Ok(())
    }

    async fn read(
        &mut self,
        address: Self::AddressType,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        let command = [0x1B, address];
        let mut operations = [Operation::Write(&command), Operation::Read(buf)];

        let _ = self.busy.wait_for_low().await;
        self.spi.transaction(&mut operations).await?;
        let _ = self.busy.wait_for_low().await;

        Ok(buf.len())
    }
}

impl<SPI, BUSY> Interface<SPI, BUSY> {
    pub const fn new(spi: SPI, busy: BUSY) -> Self {
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
