use core::convert::Infallible;

use crate::ll::{self, PacketType};

pub mod irq;
pub mod lora;

use embedded_hal::digital::OutputPin;
use embedded_hal_async::{delay::DelayNs, digital::Wait, spi::SpiDevice};
use irq::Irq;
pub use ll::RampTime;
use lora::{LoRaModemParams, LoRaModulationParams, LoRaPacketParams};

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Frequency {
    raw: [u8; 3],
}

impl Frequency {
    pub const fn new(freq_hz: u64) -> Self {
        const CRYSTAL_FREQ_HZ: u64 = 52_000_000u64;
        const PLL_STEPS: u64 = 2u64.pow(18);

        let freq_steps = ((freq_hz * PLL_STEPS) / CRYSTAL_FREQ_HZ) as u32;
        let array = freq_steps.to_be_bytes();

        Self {
            raw: [array[1], array[2], array[3]],
        }
    }

    pub const fn from_bytes(raw: [u8; 3]) -> Self {
        Self { raw }
    }

    pub fn as_bytes(&self) -> [u8; 3] {
        self.raw
    }
}

impl Default for Frequency {
    fn default() -> Self {
        Self::new(2_440_000_000)
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TxParams {
    pub power: u8,
    pub ramp_time: ll::RampTime,
}

pub struct SX128X<T, BUSY, DIO, NRESET, DELAY>
where
    T: SpiDevice,
    BUSY: Wait<Error = Infallible>,
    DIO: Wait<Error = Infallible>,
    NRESET: OutputPin<Error = Infallible>,
    DELAY: DelayNs,
{
    ll: ll::Device<ll::Interface<T, BUSY>>,
    nreset: NRESET,
    dio1: DIO,
    delay: DELAY,
}

impl<
    T: SpiDevice<Error = E>,
    BUSY: Wait<Error = Infallible>,
    DIO: Wait<Error = Infallible>,
    NRESET: OutputPin<Error = Infallible>,
    DELAY: DelayNs,
    E,
> SX128X<T, BUSY, DIO, NRESET, DELAY>
{
    pub fn new(t: T, busy: BUSY, dio1: DIO, nreset: NRESET, delay: DELAY) -> Self {
        Self {
            ll: ll::Device::new(ll::Interface::new(t, busy)),
            nreset,
            dio1,
            delay,
        }
    }

    pub async fn reset(&mut self) {
        let _ = self.nreset.set_low();
        self.delay.delay_ms(10).await;
        let _ = self.nreset.set_high();
        self.delay.delay_ms(10).await;
    }

    pub fn ll(&mut self) -> &mut ll::Device<ll::Interface<T, BUSY>> {
        &mut self.ll
    }

    pub async fn configure(&mut self, modem: LoRaModemParams) -> Result<(), E> {
        self.set_standbyrc().await?;
        self.set_rf_frequency(modem.frequency).await?;
        self.set_packet_type(PacketType::LoRa).await?;
        self.set_modulation_params(modem.modulation_params).await?;
        self.set_packet_params(modem.packet_params).await?;
        self.set_tx_params(modem.tx_params).await?;
        Ok(())
    }

    pub async fn calibrate(&mut self) -> Result<(), E> {
        self.ll
            .calibrate()
            .dispatch_async(|cmd| {
                cmd.set_rc_64_k_enable(true);
                cmd.set_rc_13_m_enable_enable(true);
                cmd.set_pll_enable(true);
                cmd.set_adc_pulse_enable(true);
                cmd.set_adc_bulk_n_enable(true);
                cmd.set_adc_bulk_p_enable(true);
            })
            .await
    }

    pub async fn send(&mut self, buf: &[u8]) -> Result<(), E> {
        self.set_buffer_base_address().await?;

        // TODO bounds check.
        self.ll.tx_buffer().write_all_async(buf).await?;
        info!("Buffer written");

        let status = self.ll.get_status().dispatch_async().await?;
        info!("Status: {}", status);

        let irq = Irq::TxDone | Irq::RxTxTimeout | Irq::CrcError; // TODO why CRC_ERROR?

        self.ll
            .set_dio_irq_params()
            .dispatch_async(|cmd| {
                cmd.set_irq_mask(irq.bits());
                cmd.set_dio_1_mask(irq.bits());
                cmd.set_dio_2_mask(Irq::empty().bits());
                cmd.set_dio_3_mask(Irq::empty().bits());
            })
            .await?;

        info!("DIO set");
        self.ll
            .set_tx()
            .dispatch_async(|cmd| cmd.set_period_base_count(ll::TxTimeoutBaseCount::SingleMode))
            .await?;
        info!("Tx mode set");

        let status = self.ll.get_status().dispatch_async().await?;
        info!("Status: {}", status);

        let _ = self.dio1.wait_for_high().await;
        info!("DIO high");
        let irqs = self.ll.get_irq_status().dispatch_async().await?;

        info!("IRQS {}", irqs);

        self.ll
            .clr_irq_status()
            .dispatch_async(|cmd| {
                cmd.set_value(irqs.value());
            })
            .await
    }

    pub async fn receive(&mut self) -> Result<(), E> {
        // SetDioIrqParams
        // SetRx
        // GetPacketStatus
        // ClrIrqStatus
        // GetRxBufferStatus
        // ReadBuffer

        todo!()
    }
}

impl<
    T: SpiDevice<Error = E>,
    BUSY: Wait<Error = Infallible>,
    DIO: Wait<Error = Infallible>,
    NRESET: OutputPin<Error = Infallible>,
    DELAY: DelayNs,
    E,
> SX128X<T, BUSY, DIO, NRESET, DELAY>
{
    async fn set_standbyrc(&mut self) -> Result<(), E> {
        self.ll
            .set_standby()
            .dispatch_async(|cmd| cmd.set_standby_config(ll::StandbyConfig::StdbyRc))
            .await
    }

    async fn set_packet_type(&mut self, packet_type: PacketType) -> Result<(), E> {
        self.ll
            .set_packet_type()
            .dispatch_async(|cmd| cmd.set_value(packet_type))
            .await
    }

    async fn set_buffer_base_address(&mut self) -> Result<(), E> {
        self.ll
            .set_buffer_base_address()
            .dispatch_async(|cmd| {
                cmd.set_tx_base_address(0x00);
                cmd.set_rx_base_address(0x00);
            })
            .await
    }

    async fn set_modulation_params(
        &mut self,
        modulation_params: LoRaModulationParams,
    ) -> Result<(), E> {
        let fec = match modulation_params.spreading_factor {
            lora::LoRaSpreadingFactor::Sf5 => ll::FEC::Sf56,
            lora::LoRaSpreadingFactor::Sf6 => ll::FEC::Sf56,
            lora::LoRaSpreadingFactor::Sf7 => ll::FEC::Sf78,
            lora::LoRaSpreadingFactor::Sf8 => ll::FEC::Sf78,
            lora::LoRaSpreadingFactor::Sf9 => ll::FEC::Sf912,
            lora::LoRaSpreadingFactor::Sf10 => ll::FEC::Sf912,
            lora::LoRaSpreadingFactor::Sf11 => ll::FEC::Sf912,
            lora::LoRaSpreadingFactor::Sf12 => ll::FEC::Sf912,
        };

        let mut buf = [0u8; 4];
        buf[1..].copy_from_slice(&modulation_params.as_bytes());
        let modulation_params = u32::from_be_bytes(buf);
        self.ll
            .set_modulation_params()
            .dispatch_async(|cmd| cmd.set_mod_params(modulation_params))
            .await?;
        self.ll
            .sf_additional_configuration()
            .modify_async(|reg| reg.set_value(fec))
            .await?;
        self.ll
            .frequency_error_correction()
            .write_async(|reg| reg.set_value(0x01))
            .await?;
        Ok(())
    }

    async fn set_packet_params(&mut self, packet_params: LoRaPacketParams) -> Result<(), E> {
        let mut buf = [0u8; 8];
        buf[1..].copy_from_slice(&packet_params.as_bytes());
        let packet_params = u64::from_be_bytes(buf);

        self.ll
            .set_packet_params()
            .dispatch_async(|cmd| cmd.set_packet_params(packet_params))
            .await

        // TODO sync word
    }

    async fn set_rf_frequency(&mut self, frequency: Frequency) -> Result<(), E> {
        let mut buf = [0u8; 4];
        buf[1..].copy_from_slice(&frequency.as_bytes());
        let frequency = u32::from_be_bytes(buf);

        self.ll
            .set_rf_frequency()
            .dispatch_async(|cmd| cmd.set_value(frequency))
            .await
    }

    async fn set_tx_params(&mut self, tx_params: TxParams) -> Result<(), E> {
        self.ll
            .set_tx_params()
            .dispatch_async(|cmd| {
                cmd.set_power(tx_params.power);
                cmd.set_ramp_time(tx_params.ramp_time);
            })
            .await
    }
}
