use autd3_driver::{
    datagram::PulseWidthEncoder,
    derive::Datagram,
    error::AUTDInternalError,
    firmware::{cpu::TxDatagram, operation::OperationHandler},
};
use autd3_firmware_emulator::{cpu::params::PULSE_WIDTH_ENCODER_FLAG_END, CPUEmulator};

use itertools::Itertools;
use rand::*;

use crate::{create_geometry, send};

#[test]
fn config_pwe() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let buf: Vec<_> = (0..32768)
            .map(|_| rng.gen_range(0..=256))
            .sorted()
            .collect();
        let full_width_start = buf
            .iter()
            .enumerate()
            .find(|&(_, v)| *v == 256)
            .map(|v| v.0 as u16 * 2)
            .unwrap_or(0xFFFF);
        let d = PulseWidthEncoder::new(|_| |i| buf[i]);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(
            full_width_start,
            cpu.fpga().pulse_width_encoder_full_width_start()
        );
        assert_eq!(
            buf.into_iter().map(|v| v as u8).collect::<Vec<_>>(),
            cpu.fpga().pulse_width_encoder_table()
        );
    }

    {
        let full_width_start = 65024;
        let default_table: Vec<_> = (0..32768)
            .map(|i| {
                if i < 255 * 255 / 2 {
                    ((i as f64 / (255 * 255 / 2) as f64).asin() / std::f64::consts::PI * 512.0)
                        .round() as u16
                } else {
                    256
                }
            })
            .collect();

        let d = PulseWidthEncoder::default();

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(
            full_width_start,
            cpu.fpga().pulse_width_encoder_full_width_start()
        );

        assert_eq!(
            default_table
                .into_iter()
                .map(|v| v as u8)
                .collect::<Vec<_>>(),
            cpu.fpga().pulse_width_encoder_table()
        );
    }

    Ok(())
}

#[test]
fn config_pwe_invalid_data_size() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: Vec<_> = (0..32768)
        .map(|_| rng.gen_range(0..=256))
        .sorted()
        .collect();
    let d = PulseWidthEncoder::new(|_| |i| buf[i]);

    let gen = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(gen, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, usize::MAX)?;
    tx[0].payload[2] = 1;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::InvalidPulseWidthEncoderDataSize),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}

#[test]
fn config_pwe_incomplete_data() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: Vec<_> = (0..32768)
        .map(|_| rng.gen_range(0..=256))
        .sorted()
        .collect();
    let d = PulseWidthEncoder::new(|_| |i| buf[i]);

    let gen = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(gen, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, usize::MAX)?;
    tx[0].payload[1] |= PULSE_WIDTH_ENCODER_FLAG_END;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::IncompletePulseWidthEncoderData),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
