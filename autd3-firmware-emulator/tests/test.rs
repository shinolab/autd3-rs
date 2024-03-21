use autd3_driver::{
    autd3_device::AUTD3,
    cpu::TxDatagram,
    datagram::*,
    error::AUTDInternalError,
    geometry::{Geometry, IntoDevice, Vector3},
    operation::{NullOp, Operation, OperationHandler},
};
use autd3_firmware_emulator::{
    cpu::params::{ERR_BIT, ERR_NOT_SUPPORTED_TAG},
    CPUEmulator,
};

mod op;

pub fn create_geometry(n: usize) -> Geometry {
    Geometry::new(
        (0..n)
            .map(|i| AUTD3::new(Vector3::zeros()).into_device(i))
            .collect(),
    )
}

pub fn send_once(
    cpu: &mut CPUEmulator,
    op: &mut impl Operation,
    geometry: &Geometry,
    tx: &mut TxDatagram,
) -> Result<(), AUTDInternalError> {
    let mut op_null = NullOp::default();
    OperationHandler::pack(op, &mut op_null, geometry, tx)?;
    cpu.send(tx);
    if (cpu.ack() & ERR_BIT) == ERR_BIT {
        return Err(AUTDInternalError::firmware_err(cpu.ack()));
    }
    assert_eq!(tx[0].header.msg_id, cpu.ack());
    Ok(())
}

pub fn send(
    cpu: &mut CPUEmulator,
    op: &mut impl Operation,
    geometry: &Geometry,
    tx: &mut TxDatagram,
) -> Result<(), AUTDInternalError> {
    let mut op_null = NullOp::default();
    OperationHandler::init(op, &mut op_null, geometry)?;
    loop {
        if OperationHandler::is_finished(op, &mut op_null, geometry) {
            break;
        }
        send_once(cpu, op, geometry, tx)?;
    }
    Ok(())
}

#[test]
fn send_invalid_tag() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    tx[0].header.msg_id = 1;
    tx[0].payload[0] = 0xFF;

    cpu.send(&tx);
    assert_eq!(ERR_NOT_SUPPORTED_TAG, cpu.ack());
}

#[test]
fn send_ingore_same_data() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let (mut op, mut op_null) = Clear::new().operation()?;

    OperationHandler::init(&mut op, &mut op_null, &geometry)?;
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx)?;

    cpu.send(&tx);
    let msg_id = tx[0].header.msg_id;
    assert_eq!(cpu.ack(), tx[0].header.msg_id);

    let (mut op, mut op_null) = Synchronize::new().operation()?;
    OperationHandler::init(&mut op, &mut op_null, &geometry)?;
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx)?;
    tx[0].header.msg_id = msg_id;
    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert!(!cpu.synchronized());

    Ok(())
}

#[test]
fn send_slot_2() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let (mut op_clear, _) = Clear::new().operation()?;
    let (mut op_sync, _) = Synchronize::new().operation()?;

    OperationHandler::init(&mut op_clear, &mut op_sync, &geometry)?;
    OperationHandler::pack(&mut op_clear, &mut op_sync, &geometry, &mut tx)?;

    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx[0].header.msg_id);
    assert!(cpu.synchronized());

    Ok(())
}
