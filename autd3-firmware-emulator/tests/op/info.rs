use autd3_driver::{
    datagram::*,
    error::AUTDInternalError,
    firmware::{
        cpu::TxDatagram,
        operation::{FirmInfoOp, NullOp, OperationHandler},
        version::FirmwareVersion,
    },
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send, send_once};

#[test]
fn send_firminfo() -> anyhow::Result<()> {
    const EMULATOR_BIT: u8 = 1 << 7;

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // configure Reads FPGA Info
    {
        assert!(!cpu.reads_fpga_state());
        let (mut op, _) = ReadsFPGAState::new(|_| true).operation();
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));
        assert!(cpu.reads_fpga_state());
    }

    let mut op = FirmInfoOp::default();
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry)?;

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MAJOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(EMULATOR_BIT, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert!(cpu.reads_fpga_state());

    Ok(())
}

#[test]
fn invalid_info_type() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let mut op = FirmInfoOp::default();
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry)?;
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx)?;
    tx[0].payload[1] = 7;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::InvalidInfoType),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
