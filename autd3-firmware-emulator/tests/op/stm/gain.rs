use std::collections::HashMap;

use autd3_driver::{
    common::EmitIntensity,
    cpu::TxDatagram,
    derive::*,
    fpga::{
        GAIN_STM_BUF_SIZE_MAX, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN,
        SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
    },
    geometry::Geometry,
    operation::{GainSTMMode, GainSTMOp},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

use super::super::gain::TestGain;

fn gen_random_buf(n: usize, geometry: &Geometry) -> Vec<HashMap<usize, Vec<Drive>>> {
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| {
                                Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()))
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect()
}

#[test]
fn test_send_gain_stm_phase_intensity_full() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(GAIN_STM_BUF_SIZE_MAX, &geometry);
    let freq_div = rng.gen_range(
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
            ..=SAMPLING_FREQ_DIV_MAX,
    );
    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseIntensityFull,
        freq_div,
        None,
        None,
    );

    assert!(!cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert!(cpu.fpga().is_stm_mode());
    assert!(cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());
    assert_eq!(bufs.len(), cpu.fpga().stm_cycle());
    assert_eq!(freq_div, cpu.fpga().stm_frequency_division());
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives(gain_idx)
            .into_iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(bufs[gain_idx][&0][i], drive);
            });
    });

    Ok(())
}

#[test]
fn test_send_gain_stm_phase_intensity_full_with_idx() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(2, &geometry);
    let start_idx = Some(rng.gen_range(0..bufs.len() as u16));
    let finish_idx = Some(rng.gen_range(0..bufs.len() as u16));
    let mut op = GainSTMOp::new(
        bufs.into_iter().map(|buf| TestGain { buf }).collect(),
        GainSTMMode::PhaseIntensityFull,
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
        start_idx,
        finish_idx,
    );

    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(start_idx, cpu.fpga().stm_start_idx());
    assert_eq!(finish_idx, cpu.fpga().stm_finish_idx());

    Ok(())
}

fn send_gain_stm_phase_full(n: usize) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(n, &geometry);
    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseFull,
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
        None,
        None,
    );

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives(gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(EmitIntensity::MAX, drive.intensity());
                assert_eq!(bufs[gain_idx][&0][i].phase(), drive.phase());
            });
    });

    Ok(())
}

#[test]
fn test_send_gain_stm_phase_full() -> anyhow::Result<()> {
    send_gain_stm_phase_full(2)?;
    send_gain_stm_phase_full(3)?;
    send_gain_stm_phase_full(GAIN_STM_BUF_SIZE_MAX)?;
    Ok(())
}

fn send_gain_stm_phase_half(n: usize) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(n, &geometry);

    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseHalf,
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
        None,
        None,
    );

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives(gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, &drive)| {
                assert_eq!(EmitIntensity::MAX, drive.intensity());
                assert_eq!(
                    bufs[gain_idx][&0][i].phase().value() >> 4,
                    drive.phase().value() >> 4
                );
            });
    });

    Ok(())
}

#[test]
fn test_send_gain_stm_phase_half() -> anyhow::Result<()> {
    send_gain_stm_phase_half(2)?;
    send_gain_stm_phase_half(3)?;
    send_gain_stm_phase_half(4)?;
    send_gain_stm_phase_half(5)?;
    send_gain_stm_phase_half(GAIN_STM_BUF_SIZE_MAX)?;
    Ok(())
}
