/*
 * File: reads_fpga_info.rs
 * Project: op
 * Created Date: 17/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::{
    autd3_device::AUTD3,
    cpu::TxDatagram,
    datagram::*,
    geometry::{Geometry, IntoDevice, Vector3},
    operation::OperationHandler,
};
use autd3_firmware_emulator::CPUEmulator;

#[test]
fn send_reads_fpga_info() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    assert!(!cpu.reads_fpga_info());

    let (mut op, mut op_null) = ConfigureReadsFPGAState::new(|_| true).operation().unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(cpu.reads_fpga_info());
    assert_eq!(cpu.rx_data(), 0);

    cpu.fpga_mut().assert_thermal_sensor();
    cpu.update();
    assert_eq!(cpu.rx_data(), 0x81);

    cpu.fpga_mut().deassert_thermal_sensor();
    cpu.update();
    assert_eq!(cpu.rx_data(), 0x80);
}
