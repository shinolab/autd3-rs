/*
 * File: test.rs
 * Project: tests
 * Created Date: 13/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
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

mod op;

#[test]
fn send_invalid_tag() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    tx.header_mut(0).msg_id = 1;
    tx.payload_mut(0)[0] = 0xFF;

    cpu.send(&tx);
    assert_eq!(
        cpu.ack(),
        autd3_firmware_emulator::cpu::params::ERR_NOT_SUPPORTED_TAG
    );
}

#[test]
fn send_ingore_same_data() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let (mut op, mut op_null) = Clear::new().operation().unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    let msg_id = tx.headers().next().unwrap().msg_id;
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);

    let (mut op, mut op_null) = Synchronize::new().operation().unwrap();
    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    tx.header_mut(0).msg_id = msg_id;
    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert!(!cpu.synchronized());
}

#[test]
fn send_slot_2() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let (mut op_clear, _) = Clear::new().operation().unwrap();
    let (mut op_sync, _) = Synchronize::new().operation().unwrap();

    OperationHandler::init(&mut op_clear, &mut op_sync, &geometry).unwrap();
    OperationHandler::pack(&mut op_clear, &mut op_sync, &geometry, &mut tx).unwrap();

    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(cpu.synchronized());
}
