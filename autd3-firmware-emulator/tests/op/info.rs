use autd3_driver::{
    datagram::*,
    error::AUTDInternalError,
    firmware::{
        cpu::TxDatagram,
        operation::{FirmwareVersionType, OperationGenerator, OperationHandler},
        version::FirmwareVersion,
    },
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_firminfo() -> anyhow::Result<()> {
    use FirmwareVersionType::*;

    const EMULATOR_BIT: u8 = 1 << 7;

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // configure Reads FPGA Info
    {
        assert!(!cpu.reads_fpga_state());
        let d = ReadsFPGAState::new(|_| true);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
        assert!(cpu.reads_fpga_state());
    }

    send(&mut cpu, CPUMajor, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MAJOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, CPUMinor, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FPGAMajor, &geometry, &mut tx)?;
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FPGAMinor, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FPGAFunctions, &geometry, &mut tx)?;
    assert_eq!(EMULATOR_BIT, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, Clear, &geometry, &mut tx)?;
    assert!(cpu.reads_fpga_state());

    Ok(())
}

#[test]
fn invalid_info_type() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = FirmwareVersionType::CPUMajor;
    let (op, op_null) = d.operation_generator(&geometry)?.generate(&geometry[0]);

    OperationHandler::pack(&mut [(op, op_null)], &geometry, &mut tx, false)?;
    tx[0].payload_mut()[1] = 7;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::InvalidInfoType),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
