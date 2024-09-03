use autd3_driver::{
    autd3_device::AUTD3,
    datagram::*,
    error::AUTDInternalError,
    firmware::{cpu::TxDatagram, operation::OperationHandler},
    geometry::{Geometry, IntoDevice, Vector3},
};
use autd3_firmware_emulator::{
    cpu::params::{ERR_BIT, ERR_INVALID_MSG_ID, ERR_NOT_SUPPORTED_TAG},
    CPUEmulator,
};

mod op;

pub fn create_geometry(n: usize) -> Geometry {
    Geometry::new(
        (0..n)
            .map(|i| {
                AUTD3::new(Vector3::zeros())
                    .into_device(i as _, (i * AUTD3::NUM_TRANS_IN_UNIT) as _)
            })
            .collect(),
    )
}

pub fn send(
    cpu: &mut CPUEmulator,
    d: impl Datagram,
    geometry: &Geometry,
    tx: &mut TxDatagram,
) -> Result<(), AUTDInternalError> {
    let _timeout = d.timeout();
    let parallel = geometry.num_devices() > d.parallel_threshold().unwrap_or(4);
    let generator = d.operation_generator(geometry)?;
    let mut op = OperationHandler::generate(generator, geometry);
    loop {
        if OperationHandler::is_done(&op) {
            break;
        }
        OperationHandler::pack(&mut op, geometry, tx, parallel)?;
        cpu.send(tx);
        if (cpu.rx().ack() & ERR_BIT) == ERR_BIT {
            return Err(AUTDInternalError::firmware_err(cpu.rx().ack()));
        }
        assert_eq!(tx[0].header.msg_id, cpu.rx().ack());
    }
    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_invalid_tag() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    tx[0].header.msg_id = 1;
    tx[0].payload[0] = 0xFF;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::firmware_err(ERR_NOT_SUPPORTED_TAG)),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_invalid_msg_id() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    tx[0].header.msg_id = 0x80;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::firmware_err(ERR_INVALID_MSG_ID)),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_ingore_same_data() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = Clear::new();
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, false)?;
    cpu.send(&tx);
    let msg_id = tx[0].header.msg_id;
    assert_eq!(cpu.rx().ack(), tx[0].header.msg_id);

    let d = Synchronize::new();
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, false)?;
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

    let d = (Clear::new(), Synchronize::new());
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, false)?;

    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert_eq!(cpu.rx().ack(), tx[0].header.msg_id);
    assert!(cpu.synchronized());

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_slot_2_err() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = (Clear::new(), Synchronize::new());
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, false)?;

    let slot2_offset = tx[0].header.slot_2_offset as usize;
    tx[0].payload[slot2_offset] = 0xFF;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::firmware_err(ERR_NOT_SUPPORTED_TAG)),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
