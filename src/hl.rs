use crate::ll;

pub mod lora;

pub use ll::RampTime;

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "defmt-1", derive(defmt::Format))]
pub struct Frequency {
    raw: [u8; 3],
}

impl Frequency {
    pub const fn new(freq_hz: u32) -> Self {
        const CRYSTAL_FREQ_HZ: u64 = 52_000_000u64;
        const PLL_STEPS: u64 = 2u64.pow(18);

        let freq_steps = ((freq_hz as u64 * PLL_STEPS) / CRYSTAL_FREQ_HZ) as u32;
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
#[cfg_attr(feature = "defmt-1", derive(defmt::Format))]
pub struct TxParams {
    pub power: u8,
    pub ramp_time: ll::RampTime,
}

pub struct SX128X<T, BUSY> {
    ll: ll::Device<ll::Interface<T, BUSY>>,
}

impl<T, BUSY> SX128X<T, BUSY> {
    pub const fn new(t: T, busy: BUSY) -> Self {
        Self {
            ll: ll::Device::new(ll::Interface::new(t, busy)),
        }
    }
}
