use embedded_hal_mock::eh1::spi::Mock;

use crate::{
    hl::{
        lora::{LoRaCrc, LoRaHeader, LoRaIq, LoRaModulationParams, LoRaPreambleLength},
        *,
    },
    ll,
    test::*,
};

const fn freq_reference(freq: u32) -> [u8; 3] {
    let val = freq as f64 / 198.3642578125;
    let val = val as u32;
    [(val >> 16) as u8, (val >> 8) as u8, val as u8]
}

#[test]
fn frequency() {
    assert_eq!(
        Frequency::new(2_400_000_000).as_bytes(),
        freq_reference(2_400_000_000)
    );
    assert_eq!(
        Frequency::new(2_440_000_000).as_bytes(),
        freq_reference(2_440_000_000)
    );
    assert_eq!(
        Frequency::new(2_485_000_000).as_bytes(),
        freq_reference(2_485_000_000)
    );
    assert_eq!(
        Frequency::new(2_405_000_000).as_bytes(),
        &0xB90000u32.to_be_bytes()[1..]
    );
    assert_eq!(
        freq_reference(2_405_000_000),
        &0xB90000u32.to_be_bytes()[1..]
    )
}

/// Test based on a capture of the competitor radio_sx128x crate working on a test device.
#[test]
fn capture_tx() {
    let expectations = [
        reg_r(0x153, &[0xA9, 0xB7]),
        cmd_w(0x80, &[0x00]),
        cmd_w(0x96, &[0x01]),
        cmd_w(0x80, &[0x00]),
        cmd_w(0x86, &[0xB9, 0x00, 0x00]),
        cmd_w(0x8A, &[0x01]),
        cmd_w(0x8B, &[0xC0, 0x34, 0x01]),
        // SF additional configuration that is not in the impl of radio_sx128x
        reg_r(0x925, &[0x00]),
        reg_w(0x925, &[0x32]),
        reg_w(0x93C, &[0x01]),
        cmd_w(0x8C, &[0x08, 0x00, 0x20, 0x20, 0x40, 0x00, 0x00]),
        cmd_w(0x8E, &[0x16, 0xE0]),
        cmd_w(0x89, &[0x3F]),
        // // After a while
        cmd_w(0x80, &[0x00]),
        cmd_w(0x86, &[0xB9, 0x00, 0x00]),
        cmd_w(0x8A, &[0x01]),
        cmd_w(0x8B, &[0xC0, 0x34, 0x01]),
        // SF additional configuration that is not in the impl of radio_sx128x
        reg_r(0x925, &[0x00]),
        reg_w(0x925, &[0x32]),
        reg_w(0x93C, &[0x01]),
        cmd_w(0x8C, &[0x08, 0x00, 0x20, 0x20, 0x40, 0x00, 0x00]),
        cmd_w(0x8E, &[0x16, 0xE0]),
        cmd_w(0x8F, &[0x00, 0x00]),
        buf_w(0x00, &[0x00; 16]),
        cmd_r(0xC0, &[0xC3]),
        cmd_w(0x8D, &[0x40, 0x41, 0x40, 0x41, 0x00, 0x00, 0x00, 0x00]),
        cmd_w(0x83, &[0x00, 0x00, 0x00]),
        cmd_r(0xC0, &[0xC3]),
        // // After polling for a while
        cmd_r(0x15, &[0x00, 0x01]),
        cmd_w(0x97, &[0x00, 0x01]),
        cmd_r(0xC0, &[0x43]),
    ];
    let mut spi = Mock::new(expectations.iter().flatten());
    let mut hl = hl::SX128X::new(&mut spi, MockWait, MockWait, MockOutput, MockDelay);

    embassy_futures::block_on(async {
        {
            // TODO make HL equivalents.
            let ll = hl.ll();

            let fw = ll.firmware_versions().read_async().await.unwrap().value();
            assert_eq!(fw, ll::FirmwareVersion::Version2);

            ll.set_standby()
                .dispatch_async(|cmd| cmd.set_standby_config(ll::StandbyConfig::StdbyRc))
                .await
                .unwrap();

            ll.set_regulator_mode()
                .dispatch_async(|cmd| cmd.set_regulator_type(ll::RegulatorType::DcDc))
                .await
                .unwrap();
        }

        let params = hl::lora::LoRaModemParams {
            frequency: Frequency::new(2_405_000_000),
            tx_params: TxParams {
                power: 22,
                ramp_time: ll::RampTime::RadioRamp20Us,
            },
            modulation_params: LoRaModulationParams {
                spreading_factor: hl::lora::LoRaSpreadingFactor::Sf12,
                bandwidth: hl::lora::LoRaBandwidth::Bw200kHz,
                coding_rate: hl::lora::LoRaCodingRate::Cr4_5,
            },
            packet_params: hl::lora::LoRaPacketParams {
                preamble_length: LoRaPreambleLength {
                    mantissa: 8,
                    exponenta: 0,
                },
                header_type: LoRaHeader::Explicit,
                payload_length: 32,
                crc_mode: LoRaCrc::Enabled,
                invert_iq: LoRaIq::Normal,
                sync_word: 0x42,
            },
        };
        hl.configure(params).await.unwrap();

        hl.calibrate().await.unwrap();

        hl.configure(params).await.unwrap();

        hl.send(&[0x00; 16]).await.unwrap();

        {
            let status = hl.ll().get_status().dispatch_async().await.unwrap();
            assert_eq!(status.circuit_mode(), Ok(ll::CircuitMode::StdbyRc));
        }
    });

    spi.done();
}
