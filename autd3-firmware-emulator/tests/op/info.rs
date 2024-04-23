use autd3_driver::{
    datagram::*,
    firmware::{
        cpu::TxDatagram,
        firmware_version::FirmwareInfo,
        operation::{FirmInfoOp, NullOp, OperationHandler},
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
        let (mut op, _) = ConfigureReadsFPGAState::new(|_| true).operation()?;
        send(&mut cpu, &mut op, &geometry, &mut tx)?;
        assert!(cpu.reads_fpga_state());
    }

    let mut op = FirmInfoOp::default();
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry)?;

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(FirmwareInfo::LATEST_VERSION_NUM_MAJOR, cpu.rx_data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(FirmwareInfo::LATEST_VERSION_NUM_MINOR, cpu.rx_data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(FirmwareInfo::LATEST_VERSION_NUM_MINOR, cpu.rx_data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!(EMULATOR_BIT, cpu.rx_data());
    assert!(!cpu.reads_fpga_state());

    send_once(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert!(cpu.reads_fpga_state());

    Ok(())
}

#[test]
#[should_panic(expected = "Unsupported firmware info type")]
fn send_firminfo_should_panic() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let mut op = FirmInfoOp::default();
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    tx[0].payload[1] = 7;
    cpu.send(&tx);
}
