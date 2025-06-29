use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    environment::Environment,
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    datagram::*,
    error::AUTDDriverError,
    firmware::{
        driver::{Driver, OperationHandler},
        v12_1::{V12_1, cpu::check_firmware_err, operation::OperationGenerator},
        version::FirmwareVersion,
    },
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[test]
fn send_firminfo() -> anyhow::Result<()> {
    use FirmwareVersionType::*;

    const EMULATOR_BIT: u8 = 1 << 7;

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    // configure Reads FPGA Info
    {
        assert!(!cpu.reads_fpga_state());
        let d = ReadsFPGAState::new(|_| true);
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
        assert!(cpu.reads_fpga_state());
    }

    send(&mut msg_id, &mut cpu, CPUMajor, &mut geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MAJOR.0, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut msg_id, &mut cpu, CPUMinor, &mut geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR.0, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut msg_id, &mut cpu, FPGAMajor, &mut geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MAJOR.0, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut msg_id, &mut cpu, FPGAMinor, &mut geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR.0, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut msg_id, &mut cpu, FPGAFunctions, &mut geometry, &mut tx)?;
    assert_eq!(EMULATOR_BIT, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut msg_id, &mut cpu, Clear, &mut geometry, &mut tx)?;
    assert!(cpu.reads_fpga_state());

    Ok(())
}

#[test]
fn invalid_info_type() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0);

    let d = FirmwareVersionType::CPUMajor;
    let operations = d
        .operation_generator(
            &geometry,
            &Environment::new(),
            &DeviceFilter::all_enabled(),
            &V12_1.firmware_limits(),
        )?
        .generate(&geometry[0]);

    OperationHandler::pack(msg_id, &mut [operations], &geometry, &mut tx, false)?;
    tx[0].payload_mut()[1] = 7;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDDriverError::InvalidInfoType),
        check_firmware_err(cpu.rx().ack())
    );

    Ok(())
}
