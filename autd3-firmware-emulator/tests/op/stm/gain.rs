/*
 * File: gain.rs
 * Project: stm
 * Created Date: 17/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use autd3_driver::{
    autd3_device::AUTD3,
    common::EmitIntensity,
    cpu::TxDatagram,
    derive::*,
    fpga::{GAIN_STM_BUF_SIZE_MAX, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
    geometry::{Geometry, IntoDevice, Vector3},
    operation::{GainSTMMode, GainSTMOp, NullOp, OperationHandler},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use super::super::gain::TestGain;

#[test]
fn send_gain_stm_phase_intensity_full() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_BUF_SIZE_MAX)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| Drive {
                                phase: Phase::new(rng.gen_range(0..=u8::MAX)),
                                intensity: EmitIntensity::new(rng.gen_range(0..=u8::MAX)),
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect();
    let gains: Vec<_> = bufs
        .iter()
        .map(|buf| TestGain { buf: buf.clone() })
        .collect();
    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let start_idx = Some(rng.gen_range(0..gains.len() as u16));
    let finish_idx = Some(rng.gen_range(0..gains.len() as u16));
    let mut op = GainSTMOp::new(
        gains,
        GainSTMMode::PhaseIntensityFull,
        freq_div,
        start_idx,
        finish_idx,
    );
    let mut op_null = NullOp::default();

    assert!(!cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    loop {
        if OperationHandler::is_finished(&mut op, &mut op_null, &geometry) {
            break;
        }
        OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
        cpu.send(&tx);
        assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    }
    assert!(cpu.fpga().is_stm_mode());
    assert!(cpu.fpga().is_stm_gain_mode());
    assert_eq!(cpu.fpga().stm_start_idx(), start_idx);
    assert_eq!(cpu.fpga().stm_finish_idx(), finish_idx);
    assert_eq!(cpu.fpga().stm_cycle(), bufs.len());
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .intensities_and_phases(gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, &(intensity, phase))| {
                assert_eq!(intensity, bufs[gain_idx][&0][i].intensity.value());
                assert_eq!(phase, bufs[gain_idx][&0][i].phase.value());
            });
    });

    let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..2)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| Drive {
                                phase: Phase::new(rng.gen_range(0..=u8::MAX)),
                                intensity: EmitIntensity::new(rng.gen_range(0..=u8::MAX)),
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect();
    let gains: Vec<_> = bufs.into_iter().map(|buf| TestGain { buf }).collect();
    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let start_idx = None;
    let finish_idx = None;
    let mut op = GainSTMOp::new(
        gains,
        GainSTMMode::PhaseIntensityFull,
        freq_div,
        start_idx,
        finish_idx,
    );
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert!(OperationHandler::is_finished(
        &mut op,
        &mut op_null,
        &geometry
    ));
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(cpu.fpga().is_stm_mode());
    assert!(cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());
    assert_eq!(cpu.fpga().stm_cycle(), 2);
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
}

#[test]
fn send_gain_stm_phase_full() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_BUF_SIZE_MAX)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| Drive {
                                phase: Phase::new(rng.gen_range(0..=u8::MAX)),
                                intensity: EmitIntensity::new(rng.gen_range(0..=u8::MAX)),
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect();
    let gains: Vec<_> = bufs
        .iter()
        .map(|buf| TestGain { buf: buf.clone() })
        .collect();
    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let start_idx = Some(rng.gen_range(0..gains.len() as u16));
    let finish_idx = Some(rng.gen_range(0..gains.len() as u16));
    let mut op = GainSTMOp::new(
        gains,
        GainSTMMode::PhaseFull,
        freq_div,
        start_idx,
        finish_idx,
    );
    let mut op_null = NullOp::default();

    assert!(!cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    loop {
        if OperationHandler::is_finished(&mut op, &mut op_null, &geometry) {
            break;
        }
        OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
        cpu.send(&tx);
        assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    }
    assert!(cpu.fpga().is_stm_mode());
    assert!(cpu.fpga().is_stm_gain_mode());
    assert_eq!(cpu.fpga().stm_start_idx(), start_idx);
    assert_eq!(cpu.fpga().stm_finish_idx(), finish_idx);
    assert_eq!(cpu.fpga().stm_cycle(), bufs.len());
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .intensities_and_phases(gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, &(intensity, phase))| {
                assert_eq!(intensity, 0xFF);
                assert_eq!(phase, bufs[gain_idx][&0][i].phase.value());
            });
    });

    let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..2)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| Drive {
                                phase: Phase::new(rng.gen_range(0..=u8::MAX)),
                                intensity: EmitIntensity::new(rng.gen_range(0..=u8::MAX)),
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect();
    let gains: Vec<_> = bufs.into_iter().map(|buf| TestGain { buf }).collect();
    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let start_idx = None;
    let finish_idx = None;
    let mut op = GainSTMOp::new(
        gains,
        GainSTMMode::PhaseFull,
        freq_div,
        start_idx,
        finish_idx,
    );
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert!(OperationHandler::is_finished(
        &mut op,
        &mut op_null,
        &geometry
    ));
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(cpu.fpga().is_stm_mode());
    assert!(cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());
    assert_eq!(cpu.fpga().stm_cycle(), 2);
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
}

#[test]
fn send_gain_stm_phase_half() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_BUF_SIZE_MAX)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| Drive {
                                phase: Phase::new(rng.gen_range(0..=u8::MAX)),
                                intensity: EmitIntensity::new(rng.gen_range(0..=u8::MAX)),
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect();
    let gains: Vec<_> = bufs
        .iter()
        .map(|buf| TestGain { buf: buf.clone() })
        .collect();
    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let start_idx = Some(rng.gen_range(0..gains.len() as u16));
    let finish_idx = Some(rng.gen_range(0..gains.len() as u16));
    let mut op = GainSTMOp::new(
        gains,
        GainSTMMode::PhaseHalf,
        freq_div,
        start_idx,
        finish_idx,
    );
    let mut op_null = NullOp::default();

    assert!(!cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    loop {
        if OperationHandler::is_finished(&mut op, &mut op_null, &geometry) {
            break;
        }
        OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
        cpu.send(&tx);
        assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    }
    assert!(cpu.fpga().is_stm_mode());
    assert!(cpu.fpga().is_stm_gain_mode());
    assert_eq!(cpu.fpga().stm_start_idx(), start_idx);
    assert_eq!(cpu.fpga().stm_finish_idx(), finish_idx);
    assert_eq!(cpu.fpga().stm_cycle(), bufs.len());
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .intensities_and_phases(gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, &(intensity, phase))| {
                assert_eq!(intensity, 0xFF);
                assert_eq!(phase >> 4, bufs[gain_idx][&0][i].phase.value() >> 4);
            });
    });

    let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..4)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| Drive {
                                phase: Phase::new(rng.gen_range(0..=u8::MAX)),
                                intensity: EmitIntensity::new(rng.gen_range(0..=u8::MAX)),
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect();
    let gains: Vec<_> = bufs.into_iter().map(|buf| TestGain { buf }).collect();
    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let start_idx = None;
    let finish_idx = None;
    let mut op = GainSTMOp::new(
        gains,
        GainSTMMode::PhaseHalf,
        freq_div,
        start_idx,
        finish_idx,
    );
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert!(OperationHandler::is_finished(
        &mut op,
        &mut op_null,
        &geometry
    ));
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(cpu.fpga().is_stm_mode());
    assert!(cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());
    assert_eq!(cpu.fpga().stm_cycle(), 4);
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
}
