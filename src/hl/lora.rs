use super::{Frequency, TxParams};

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaSpreadingFactor {
    Sf5 = 0x50,
    Sf6 = 0x60,
    Sf7 = 0x70,
    #[default]
    Sf8 = 0x80,
    Sf9 = 0x90,
    Sf10 = 0xA0,
    Sf11 = 0xB0,
    Sf12 = 0xC0,
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaBandwidth {
    #[default]
    Bw200kHz = 0x34,
    Bw400kHz = 0x26,
    Bw800kHz = 0x18,
    Bw1600kHz = 0x0A,
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaCodingRate {
    #[default]
    Cr4_5 = 0x01,
    Cr4_6 = 0x02,
    Cr4_7 = 0x03,
    Cr4_8 = 0x04,
    CrLi4_5 = 0x05,
    CrLi4_6 = 0x06,
    CrLi4_7 = 0x07,
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoRaModulationParams {
    pub spreading_factor: LoRaSpreadingFactor,
    pub bandwidth: LoRaBandwidth,
    pub coding_rate: LoRaCodingRate,
}

impl LoRaModulationParams {
    pub(crate) fn as_bytes(&self) -> [u8; 3] {
        [
            self.spreading_factor as u8,
            self.bandwidth as u8,
            self.coding_rate as u8,
        ]
    }
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaCrc {
    #[default]
    Enabled = 0x20,
    Disabled = 0x00,
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaIq {
    #[default]
    Normal = 0x40,
    Inverted = 0x00,
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaHeader {
    #[default]
    Explicit = 0x00,
    Implicit = 0x80,
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoRaPacketParams {
    pub preamble_length: LoRaPreambleLength,
    pub header_type: LoRaHeader,
    pub payload_length: u8,
    pub crc_mode: LoRaCrc,
    pub invert_iq: LoRaIq,
    pub sync_word: u8,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoRaPreambleLength {
    pub mantissa: u8,
    pub exponenta: u8,
}

impl LoRaPreambleLength {
    pub fn value(&self) -> u8 {
        self.mantissa & 0x0f | (self.exponenta << 4)
    }
}

impl Default for LoRaPreambleLength {
    fn default() -> Self {
        Self {
            mantissa: 1,
            exponenta: 3,
        }
    }
}

impl LoRaPacketParams {
    pub(crate) fn as_bytes(&self) -> [u8; 7] {
        [
            self.preamble_length.value(),
            self.header_type as u8,
            self.payload_length,
            self.crc_mode as u8,
            self.invert_iq as u8,
            0x00,
            0x00,
        ]
    }
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoRaModemParams {
    pub frequency: Frequency,
    pub tx_params: TxParams,
    pub modulation_params: LoRaModulationParams,
    pub packet_params: LoRaPacketParams,
}
