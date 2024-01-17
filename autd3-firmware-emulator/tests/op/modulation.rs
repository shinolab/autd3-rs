/*
 * File: modulation.rs
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
    common::EmitIntensity,
    cpu::TxDatagram,
    fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
    geometry::{Geometry, IntoDevice, Vector3},
    operation::{ModulationOp, NullOp, OperationHandler},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

#[test]
fn send_mod() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let m: Vec<_> = (0..65536).map(|_| EmitIntensity::new(rng.gen())).collect();
    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let mut op = ModulationOp::new(m.clone(), freq_div);
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    loop {
        if OperationHandler::is_finished(&mut op, &mut op_null, &geometry) {
            break;
        }
        OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
        cpu.send(&tx);
        assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    }
    assert_eq!(cpu.fpga().modulation_cycle(), m.len());
    cpu.fpga()
        .modulation()
        .iter()
        .zip(m.iter())
        .for_each(|(&a, b)| {
            assert_eq!(a, b.value());
        });
    assert_eq!(cpu.fpga().modulation_frequency_division(), freq_div);
}
