use autd3_driver::{
    derive::NullOp,
    error::AUTDInternalError,
    firmware::{
        cpu::TxDatagram,
        operation::{ConfigurePulseWidthEncoderOp, OperationHandler},
    },
};
use autd3_firmware_emulator::{cpu::params::PULSE_WIDTH_ENCODER_FLAG_END, CPUEmulator};

use rand::*;

use crate::{create_geometry, send};

#[test]
fn config_pwe() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: Vec<_> = (0..65536).map(|_| rng.gen()).collect();
    let full_width_start = buf
        .iter()
        .enumerate()
        .find(|&(_, v)| *v == 256)
        .map(|v| v.0 as u16)
        .unwrap_or(0xFFFF);
    let mut op = ConfigurePulseWidthEncoderOp::new(buf.clone());

    assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

    assert_eq!(
        full_width_start,
        cpu.fpga().pulse_width_encoder_full_width_start()
    );
    assert_eq!(
        buf.into_iter().map(|v| v as u8).collect::<Vec<_>>(),
        cpu.fpga().pulse_width_encoder_table()
    );

    Ok(())
}

#[test]
fn config_pwe_invalid_table_size() {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: Vec<_> = (0..65535).map(|_| rng.gen()).collect();
    let mut op = ConfigurePulseWidthEncoderOp::new(buf.clone());

    assert_eq!(
        Err(autd3_driver::error::AUTDInternalError::InvalidPulseWidthEncoderTableSize(65535)),
        send(&mut cpu, &mut op, &geometry, &mut tx)
    );
}

#[test]
fn config_pwe_invalid_data_size() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: Vec<_> = (0..65536).map(|_| rng.gen()).collect();
    let mut op = ConfigurePulseWidthEncoderOp::new(buf.clone());
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry)?;
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx)?;
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

    let buf: Vec<_> = (0..65536).map(|_| rng.gen()).collect();
    let mut op = ConfigurePulseWidthEncoderOp::new(buf);
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry)?;
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx)?;
    tx[0].payload[1] |= PULSE_WIDTH_ENCODER_FLAG_END;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::IncompletePulseWidthEncoderData),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
