use autd3_core::{datagram::Operation, link::MsgId};
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::*,
    error::AUTDDriverError,
    firmware::{
        cpu::TxMessage,
        operation::{OperationGenerator, OperationHandler},
    },
    geometry::{Geometry, Point3},
};
use autd3_firmware_emulator::{CPUEmulator, cpu::params::ERR_BIT};
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

pub fn send<D>(
    msg_id: &mut MsgId,
    cpu: &mut CPUEmulator,
    d: D,
    geometry: &Geometry,
    tx: &mut [TxMessage],
) -> Result<(), AUTDDriverError>
where
    D: Datagram,
    AUTDDriverError: From<D::Error>,
    D::G: OperationGenerator,
    AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
        + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
{
    let option = d.option();
    let parallel = geometry.num_devices() > option.parallel_threshold;
    let generator = d.operation_generator(geometry)?;
    let mut op = OperationHandler::generate(generator, geometry);
    let mut sent_flags = vec![false; geometry.len()];
    loop {
        if OperationHandler::is_done(&op) {
            break;
        }
        msg_id.increment();
        OperationHandler::pack(*msg_id, &mut op, geometry, &mut sent_flags, tx, parallel)?;
        cpu.send(tx);
        if (cpu.rx().ack() & ERR_BIT) == ERR_BIT {
            return Err(AUTDDriverError::firmware_err(cpu.rx().ack()));
        }
        assert_eq!(tx[0].header.msg_id.get(), cpu.rx().ack());
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
        autd3_driver::firmware::cpu::check_firmware_err(&cpu.rx())
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
        autd3_driver::firmware::cpu::check_firmware_err(&cpu.rx())
    );
}

#[test]
fn send_ignore_same_data() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut sent_flags = vec![false; 1];
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0x10);

    let d = Clear::new();
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut sent_flags, &mut tx, false)?;
    cpu.send(&tx);
    assert_eq!(cpu.rx().ack(), tx[0].header.msg_id.get());

    let d = Synchronize::new();
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut sent_flags, &mut tx, false)?;
    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert!(!cpu.synchronized());

    Ok(())
}

#[test]
fn send_slot_2_unsafe() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut sent_flags = vec![false; 1];
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0x12);

    let d = (Clear::new(), Synchronize::new());
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut sent_flags, &mut tx, false)?;

    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert_eq!(cpu.rx().ack(), tx[0].header.msg_id.get());
    assert!(cpu.synchronized());

    Ok(())
}

#[test]
fn send_slot_2_err() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut sent_flags = vec![false; 1];
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0);

    let d = (Clear::new(), Synchronize::new());
    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut sent_flags, &mut tx, false)?;

    let slot2_offset = tx[0].header.slot_2_offset as usize;
    tx[0].payload_mut()[slot2_offset] = 0xFF;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDDriverError::NotSupportedTag),
        autd3_driver::firmware::cpu::check_firmware_err(&cpu.rx())
    );

    Ok(())
}
