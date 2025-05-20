use core::convert::Infallible;

use embedded_hal::digital::ErrorType;
use embedded_hal_async::digital::Wait;
use embedded_hal_mock::eh1::spi::Mock;

use crate::{hl::irq::Irq, ll, test::*};

#[test]
fn command() {
    let expectations = [cmd_w(0x84, &[0b10])];
    let mut spi = Mock::new(expectations.iter().flatten());
    let mut ll = ll::Device::new(ll::Interface::new(&mut spi, MockWait));

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
    let mut ll = ll::Device::new(ll::Interface::new(&mut spi, MockWait));

    embassy_futures::block_on(async {
        let fw = ll.firmware_versions().read_async().await.unwrap().value();
        assert_eq!(fw, ll::FirmwareVersion::Version1);
    });

    spi.done();
}

#[test]
fn set_dio() {
    let expectations = [cmd_w(
        0x8D,
        &[0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    )];
    let mut spi = Mock::new(expectations.iter().flatten());
    let mut ll = ll::Device::new(ll::Interface::new(&mut spi, MockWait));

    embassy_futures::block_on(async {
        let irq = Irq::TxDone;

        ll.set_dio_irq_params()
            .dispatch_async(|cmd| {
                cmd.set_irq_mask(irq.to_reg());
                cmd.set_dio_1_mask(irq.to_reg());
                cmd.set_dio_2_mask(Irq::empty().bits());
                cmd.set_dio_3_mask(Irq::empty().bits());
            })
            .await
            .unwrap();
    });

    spi.done();
}

/// Test based on a capture of the competitor radio_sx128x crate working on a test device.
#[test]
fn capture_tx() {
    let expectations = [
        reg_r(0x153, &[0xA9, 0xB7]),
        cmd_w(0x80, &[0x00]),
        cmd_w(0x96, &[0x01]),
        cmd_w(0x86, &[0xB9, 0x00, 0x00]),
        cmd_w(0x8A, &[0x01]),
        cmd_w(0x8B, &[0xC0, 0x34, 0x01]),
        cmd_w(0x8C, &[0x08, 0x00, 0x20, 0x20, 0x40, 0x00, 0x00]),
        cmd_w(0x8E, &[0x16, 0xE0]),
        cmd_w(0x89, &[0x3F]),
        // After a while
        cmd_r(0xC0, &[0x43]),
        cmd_w(0x86, &[0xB9, 0x00, 0x00]),
        cmd_w(0x8C, &[0x08, 0x00, 0x20, 0x20, 0x40, 0x00, 0x00]),
        cmd_w(0x8F, &[0x00, 0x00]),
        buf_w(0x00, &[0x00; 16]),
        cmd_w(0x8D, &[0x40, 0x41, 0x40, 0x41, 0x00, 0x00, 0x00, 0x00]),
        cmd_w(0x83, &[0x00, 0x00, 0x00]),
        cmd_r(0xC0, &[0xC3]),
        cmd_r(0x15, &[0x00, 0x00]),
        cmd_r(0xC0, &[0xC3]),
        // After polling for a while
        cmd_r(0x15, &[0x00, 0x01]),
        cmd_w(0x97, &[0x00, 0x01]),
        cmd_r(0xC0, &[0x43]),
    ];
    let mut spi = Mock::new(expectations.iter().flatten());
    let mut ll = ll::Device::new(ll::Interface::new(&mut spi, MockWait));

    embassy_futures::block_on(async {
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

        ll.set_rf_frequency()
            .dispatch_async(|cmd| cmd.set_value(0xB90000)) // 2_405_000_000
            .await
            .unwrap();

        ll.set_packet_type()
            .dispatch_async(|cmd| cmd.set_value(ll::PacketType::LoRa))
            .await
            .unwrap();

        ll.set_modulation_params()
            .dispatch_async(|cmd| cmd.set_mod_params(0xC03401))
            .await
            .unwrap();

        ll.set_packet_params()
            .dispatch_async(|cmd| cmd.set_packet_params(0x08002020400000))
            .await
            .unwrap();

        ll.set_tx_params()
            .dispatch_async(|cmd| {
                cmd.set_power(0x16);
                cmd.set_ramp_time(ll::RampTime::RadioRamp20Us);
            })
            .await
            .unwrap();

        ll.calibrate()
            .dispatch_async(|cmd| {
                cmd.set_rc_64_k_enable(true);
                cmd.set_rc_13_m_enable_enable(true);
                cmd.set_pll_enable(true);
                cmd.set_adc_pulse_enable(true);
                cmd.set_adc_bulk_n_enable(true);
                cmd.set_adc_bulk_p_enable(true);
            })
            .await
            .unwrap();

        {
            let status = ll.get_status().dispatch_async().await.unwrap();
            assert_eq!(status.circuit_mode(), Ok(ll::CircuitMode::StdbyRc));
        }

        ll.set_rf_frequency()
            .dispatch_async(|cmd| cmd.set_value(0xB90000)) // 2_405_000_000
            .await
            .unwrap();

        ll.set_packet_params()
            .dispatch_async(|cmd| cmd.set_packet_params(0x08002020400000))
            .await
            .unwrap();

        ll.set_buffer_base_address()
            .dispatch_async(|cmd| {
                cmd.set_rx_base_address(0x00);
                cmd.set_tx_base_address(0x00);
            })
            .await
            .unwrap();
        {
            let mut buf = [0x00; 16];
            ll.tx_buffer().write_all_async(&mut buf).await.unwrap();
        }

        ll.set_dio_irq_params()
            .dispatch_async(|cmd| {
                cmd.set_irq_mask(0x4041);
                cmd.set_dio_1_mask(0x4041);
            })
            .await
            .unwrap();

        ll.set_tx()
            .dispatch_async(|cmd| {
                cmd.set_period_base(ll::TxTimeoutStep::Step15Us625);
                cmd.set_period_base_count(ll::TxTimeoutBaseCount::SingleMode);
            })
            .await
            .unwrap();

        {
            let status = ll.get_status().dispatch_async().await.unwrap();
            assert_eq!(status.circuit_mode(), Ok(ll::CircuitMode::Tx));
        }

        {
            let irq_status = ll.get_irq_status().dispatch_async().await.unwrap();
            assert_eq!(irq_status.value(), 0x0000u16);
        }

        {
            let status = ll.get_status().dispatch_async().await.unwrap();
            assert_eq!(status.circuit_mode(), Ok(ll::CircuitMode::Tx));
        }

        // After polling for a while
        {
            let irq_status = ll.get_irq_status().dispatch_async().await.unwrap();
            assert_eq!(irq_status.value(), 0x0001u16);
        }

        ll.clr_irq_status()
            .dispatch_async(|cmd| cmd.set_value(0x0001u16))
            .await
            .unwrap();

        {
            let status = ll.get_status().dispatch_async().await.unwrap();
            assert_eq!(status.circuit_mode(), Ok(ll::CircuitMode::StdbyRc));
        }
    });

    spi.done();
}
