/*
 * File: test.rs
 * Project: tests
 * Created Date: 13/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 14/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use autd3_derive::Gain;
use autd3_driver::{
    autd3_device::AUTD3,
    common::EmitIntensity,
    cpu::TxDatagram,
    datagram::*,
    derive::prelude::*,
    firmware_version::{LATEST_VERSION_NUM_MAJOR, LATEST_VERSION_NUM_MINOR},
    fpga::{
        FOCUS_STM_BUF_SIZE_MAX, FOCUS_STM_FIXED_NUM_UNIT, GAIN_STM_BUF_SIZE_MAX,
        SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN,
    },
    geometry::{Geometry, IntoDevice, Vector3},
    operation::{
        ControlPoint, FirmInfoOp, FocusSTMOp, GainOp, GainSTMMode, GainSTMOp, ModulationOp, NullOp,
        OperationHandler,
    },
};
use autd3_firmware_emulator::CPUEmulator;

use num_integer::Roots;
use rand::*;

#[test]
fn send_clear() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let (mut op, mut op_null) = Clear::new().operation().unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);

    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
}

#[test]
fn send_sync() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let (mut op, mut op_null) = Synchronize::new().operation().unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    assert!(!cpu.synchronized());
    cpu.send(&tx);
    assert!(cpu.synchronized());
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
}

#[test]
fn send_reads_fpga_info() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    assert!(!cpu.reads_fpga_info());

    let (mut op, mut op_null) = ConfigureReadsFPGAInfo::new(|_| true).operation().unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();

    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(cpu.reads_fpga_info());
    assert_eq!(cpu.rx_data(), 0);

    cpu.fpga_mut().assert_thermal_sensor();
    cpu.update();
    assert_eq!(cpu.rx_data(), 0x01);

    cpu.fpga_mut().deassert_thermal_sensor();
    cpu.update();
    assert_eq!(cpu.rx_data(), 0x00);
}

#[test]
fn send_firminfo() {
    const EMULATOR_BIT: u8 = 1 << 7;

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    // configure Reads FPGA Info
    {
        assert!(!cpu.reads_fpga_info());

        let (mut op, mut op_null) = ConfigureReadsFPGAInfo::new(|_| true).operation().unwrap();

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

#[test]
fn send_mod() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    // let m: Vec<_> = (0..65536).map(|_| EmitIntensity::new(rng.gen())).collect();
    let m: Vec<_> = (0..65536)
        .map(|i| {
            if i == 32768 {
                EmitIntensity::MAX
            } else {
                EmitIntensity::MIN
            }
        })
        .collect();
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

#[test]
fn send_mod_delay() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let (mut op, mut op_null) = ConfigureModDelay::new(|_dev, tr| tr.idx() as u16)
        .operation()
        .unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    cpu.fpga()
        .mod_delays()
        .iter()
        .enumerate()
        .for_each(|(i, &d)| {
            assert_eq!(d, i as u16);
        });
}

#[test]
fn send_silencer() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    let step_intensity = rng.gen_range(1..=u16::MAX);
    let step_phase = rng.gen_range(1..=u16::MAX);
    let (mut op, mut op_null) = Silencer::new(step_intensity, step_phase)
        .unwrap()
        .operation()
        .unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert_eq!(cpu.fpga().silencer_step_intensity(), step_intensity);
    assert_eq!(cpu.fpga().silencer_step_phase(), step_phase);
}

#[derive(Gain)]
struct TestGain {
    buf: HashMap<usize, Vec<Drive>>,
}

impl Gain for TestGain {
    fn calc(
        &self,
        _: &Geometry,
        _: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(self.buf.clone())
    }
}

#[test]
fn send_gain() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let buf: HashMap<usize, Vec<Drive>> = geometry
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
        .collect();
    let g = TestGain { buf: buf.clone() };

    let mut op = GainOp::new(g);
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(!cpu.fpga().is_stm_mode());
    cpu.fpga()
        .intensities_and_phases(0)
        .iter()
        .zip(buf[&0].iter())
        .for_each(|(a, b)| {
            assert_eq!(a.0, b.intensity.value());
            assert_eq!(a.1, b.phase.value());
        });
}

#[test]
fn send_focus_stm() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let foci: Vec<_> = (0..FOCUS_STM_BUF_SIZE_MAX)
        .map(|_| {
            ControlPoint::new(Vector3::new(
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-100.0..100.0),
            ))
            .with_intensity(rng.gen::<u8>())
        })
        .collect();
    let start_idx = Some(rng.gen_range(0..foci.len()) as u16);
    let finish_idx = Some(rng.gen_range(0..foci.len()) as u16);
    let mut op = FocusSTMOp::new(foci.clone(), freq_div, start_idx, finish_idx);
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
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert_eq!(cpu.fpga().stm_start_idx(), start_idx);
    assert_eq!(cpu.fpga().stm_finish_idx(), finish_idx);
    assert_eq!(cpu.fpga().stm_cycle(), foci.len());
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
    assert_eq!(
        cpu.fpga().sound_speed(),
        (geometry[0].sound_speed * 1024.0 / 1000.0).round() as _
    );
    foci.iter().enumerate().for_each(|(focus_idx, focus)| {
        cpu.fpga()
            .intensities_and_phases(focus_idx)
            .iter()
            .enumerate()
            .for_each(|(tr_idx, &(intensity, phase))| {
                let tx =
                    (geometry[0][tr_idx].position().x / FOCUS_STM_FIXED_NUM_UNIT).floor() as i32;
                let ty =
                    (geometry[0][tr_idx].position().y / FOCUS_STM_FIXED_NUM_UNIT).floor() as i32;
                let tz =
                    (geometry[0][tr_idx].position().z / FOCUS_STM_FIXED_NUM_UNIT).floor() as i32;
                let fx = (focus.point().x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let fy = (focus.point().y / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let fz = (focus.point().z / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt() as u64;
                let q = (d << 18) / cpu.fpga().sound_speed() as u64;
                assert_eq!(phase, (q & 0xFF) as u8);
                assert_eq!(intensity, focus.intensity().value());
            })
    });

    let foci: Vec<_> = (0..2)
        .map(|_| {
            ControlPoint::new(Vector3::new(
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-100.0..100.0),
                rng.gen_range(100.0..200.0),
            ))
            .with_intensity(rng.gen::<u8>())
        })
        .collect();
    let start_idx = None;
    let finish_idx = None;
    let mut op = FocusSTMOp::new(foci.clone(), freq_div, start_idx, finish_idx);
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
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());
    assert_eq!(cpu.fpga().stm_cycle(), foci.len());
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
    assert_eq!(
        cpu.fpga().sound_speed(),
        (geometry[0].sound_speed * 1024.0 / 1000.0).round() as _
    );
}

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

#[test]
fn send_force_fan() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    assert!(!cpu.fpga().is_force_fan());

    let mut tx = TxDatagram::new(1);
    let (mut op, mut op_null) = ConfigureForceFan::new(|_dev| true).operation().unwrap();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert!(cpu.fpga().is_force_fan());

    let (mut op, mut op_null) = ConfigureForceFan::new(|_dev| false).operation().unwrap();
    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert!(!cpu.fpga().is_force_fan());
}

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

#[test]
#[should_panic(expected = "not implemented: Unsupported tag")]
fn send_invalid_tag() {
    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);
    tx.header_mut(0).msg_id = 1;
    tx.payload_mut(0)[0] = 0xFF;

    cpu.send(&tx);
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
