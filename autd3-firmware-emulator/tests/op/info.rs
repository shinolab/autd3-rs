/*
 * File: info.rs
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
    firmware_version::{LATEST_VERSION_NUM_MAJOR, LATEST_VERSION_NUM_MINOR},
    geometry::{Geometry, IntoDevice, Vector3},
    operation::{FirmInfoOp, NullOp, OperationHandler},
};
use autd3_firmware_emulator::CPUEmulator;

#[test]
fn send_firminfo() {
    const EMULATOR_BIT: u8 = 1 << 7;

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    // configure Reads FPGA Info
    {
        assert!(!cpu.reads_fpga_info());

        let (mut op, mut op_null) = ConfigureReadsFPGAState::new(|_| true).operation().unwrap();

        OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();

        OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
        cpu.send(&tx);
        assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
        assert!(cpu.reads_fpga_info());
    }

    let mut op = FirmInfoOp::default();
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(cpu.rx_data(), LATEST_VERSION_NUM_MAJOR);
    assert!(!cpu.reads_fpga_info());

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(cpu.rx_data(), LATEST_VERSION_NUM_MINOR);
    assert!(!cpu.reads_fpga_info());

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(cpu.rx_data(), LATEST_VERSION_NUM_MAJOR);
    assert!(!cpu.reads_fpga_info());

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(cpu.rx_data(), LATEST_VERSION_NUM_MINOR);
    assert!(!cpu.reads_fpga_info());

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(cpu.rx_data(), EMULATOR_BIT);
    assert!(!cpu.reads_fpga_info());

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert!(cpu.reads_fpga_info());
}

#[test]
#[should_panic(expected = "Unsupported firmware info type")]
fn send_firminfo_should_panic() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let mut op = FirmInfoOp::default();
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    tx.payload_mut(0)[1] = 7;
    cpu.send(&tx);
}
