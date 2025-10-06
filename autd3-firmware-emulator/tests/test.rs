use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    geometry::{Geometry, Point3},
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::*,
    error::AUTDDriverError,
    firmware::{
        cpu::check_firmware_err,
        operation::{Operation, OperationGenerator, OperationHandler},
    },
};
use autd3_firmware_emulator::CPUEmulator;
use zerocopy::FromZeros;

mod op;

pub fn create_geometry(n: usize) -> Geometry {
    Geometry::new(
        (0..n)
            .map(|_| {
                AUTD3 {
                    pos: Point3::origin(),
                    ..Default::default()
                }
                .into()
            })
            .collect(),
    )
}

pub fn send<'a, D>(
    msg_id: &mut MsgId,
    cpu: &mut CPUEmulator,
    d: D,
    geometry: &'a mut Geometry,
    tx: &mut [TxMessage],
) -> Result<(), AUTDDriverError>
where
    D: Datagram<'a>,
    AUTDDriverError: From<D::Error>,
    D::G: OperationGenerator<'a>,
    AUTDDriverError: From<<<D::G as OperationGenerator<'a>>::O1 as Operation<'a>>::Error>
        + From<<<D::G as OperationGenerator<'a>>::O2 as Operation<'a>>::Error>,
{
    let option = d.option();
    let parallel = geometry.num_devices() > option.parallel_threshold;
    let mut generator =
        d.operation_generator(geometry, &Environment::new(), &DeviceMask::AllEnabled)?;
    let mut op = geometry
        .iter()
        .map(|dev| generator.generate(dev))
        .collect::<Vec<_>>();
    loop {
        if OperationHandler::is_done(&op) {
            break;
        }
        msg_id.increment();
        OperationHandler::pack(*msg_id, &mut op, geometry, tx, parallel)?;
        cpu.send(tx);
        check_firmware_err(cpu.rx().ack())?;
        assert_eq!(tx[0].header.msg_id.get(), cpu.rx().ack().msg_id());
    }
    Ok(())
}

#[test]
fn send_invalid_tag() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    tx[0].header.msg_id = MsgId::new(1);
    tx[0].payload_mut()[0] = 0xFF;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDDriverError::NotSupportedTag),
        check_firmware_err(cpu.rx().ack())
    );
}

#[test]
fn send_invalid_msg_id() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    tx[0].header.msg_id = MsgId::new(0x80);

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDDriverError::InvalidMessageID),
        check_firmware_err(cpu.rx().ack())
    );
}

#[test]
fn send_ignore_same_data() -> Result<(), Box<dyn std::error::Error>> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0x0A);

    let d = Clear::new();
    let mut generator =
        d.operation_generator(&geometry, &Environment::new(), &DeviceMask::AllEnabled)?;
    let mut op = geometry
        .iter()
        .map(|dev| generator.generate(dev))
        .collect::<Vec<_>>();
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)?;
    cpu.send(&tx);
    assert_eq!(cpu.rx().ack().msg_id(), tx[0].header.msg_id.get());

    let d = Synchronize::new();
    let mut generator =
        d.operation_generator(&geometry, &Environment::new(), &DeviceMask::AllEnabled)?;
    let mut op = geometry
        .iter()
        .map(|dev| generator.generate(dev))
        .collect::<Vec<_>>();
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)?;
    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert!(!cpu.synchronized());

    Ok(())
}

#[test]
fn send_slot_2_unsafe() -> Result<(), Box<dyn std::error::Error>> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0x0E);

    let d = (Clear::new(), Synchronize::new());
    let mut generator =
        d.operation_generator(&geometry, &Environment::new(), &DeviceMask::AllEnabled)?;
    let mut op = geometry
        .iter()
        .map(|dev| generator.generate(dev))
        .collect::<Vec<_>>();
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)?;

    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert_eq!(cpu.rx().ack().msg_id(), tx[0].header.msg_id.get());
    assert!(cpu.synchronized());

    Ok(())
}

#[test]
fn send_slot_2_err() -> Result<(), Box<dyn std::error::Error>> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0);

    let d = (Clear::new(), Synchronize::new());
    let mut generator =
        d.operation_generator(&geometry, &Environment::new(), &DeviceMask::AllEnabled)?;
    let mut op = geometry
        .iter()
        .map(|dev| generator.generate(dev))
        .collect::<Vec<_>>();
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)?;

    let slot2_offset = tx[0].header.slot_2_offset as usize;
    tx[0].payload_mut()[slot2_offset] = 0xFF;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDDriverError::NotSupportedTag),
        check_firmware_err(cpu.rx().ack())
    );

    Ok(())
}
