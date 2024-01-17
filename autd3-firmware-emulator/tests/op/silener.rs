/*
 * File: silener.rs
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

use rand::*;

#[test]
fn send_silencer_fixed_update_rate() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let update_rate_intensity = rng.gen_range(1..=u16::MAX);
    let update_rate_phase = rng.gen_range(1..=u16::MAX);
    let (mut op, mut op_null) =
        ConfigureSilencer::fixed_update_rate(update_rate_intensity, update_rate_phase)
            .unwrap()
            .operation()
            .unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(
        cpu.fpga().silencer_update_rate_intensity(),
        update_rate_intensity
    );
    assert_eq!(cpu.fpga().silencer_update_rate_phase(), update_rate_phase);
    assert!(!cpu.fpga().silencer_fixed_completion_steps_mode());
}

#[test]
fn send_silencer_fixed_completion_steps() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let steps_intensity = rng.gen_range(1..=10);
    let steps_phase = rng.gen_range(1..=u16::MAX);
    let (mut op, mut op_null) =
        ConfigureSilencer::fixed_completion_steps(steps_intensity, steps_phase)
            .unwrap()
            .operation()
            .unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(
        cpu.fpga().silencer_completion_steps_intensity(),
        steps_intensity
    );
    assert_eq!(cpu.fpga().silencer_completion_steps_phase(), steps_phase);
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(cpu.silencer_strict_mode());
}

#[test]
fn send_silencer_fixed_completion_steps_permissive() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let steps_intensity = rng.gen_range(1..=u16::MAX);
    let steps_phase = rng.gen_range(1..=u16::MAX);
    let (mut op, mut op_null) =
        ConfigureSilencer::fixed_completion_steps(steps_intensity, steps_phase)
            .unwrap()
            .with_strict_mode(false)
            .operation()
            .unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(
        cpu.fpga().silencer_completion_steps_intensity(),
        steps_intensity
    );
    assert_eq!(cpu.fpga().silencer_completion_steps_phase(), steps_phase);
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict_mode());
}
