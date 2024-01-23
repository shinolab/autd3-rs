use autd3_driver::{
    autd3_device::AUTD3,
    cpu::TxDatagram,
    datagram::*,
    geometry::{Geometry, IntoDevice, Vector3},
    operation::OperationHandler,
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

#[test]
fn send_debug_output_idx() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let idx = rng.gen_range(0..geometry[0].num_transducers());
    let (mut op, mut op_null) = ConfigureDebugOutputIdx::new(|dev| Some(&dev[idx]))
        .operation()
        .unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    assert!(cpu.fpga().debug_output_idx().is_none());
    cpu.send(&tx);
    assert_eq!(cpu.fpga().debug_output_idx(), Some(idx as _));
}
