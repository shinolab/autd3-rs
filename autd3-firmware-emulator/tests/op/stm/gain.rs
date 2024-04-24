use std::collections::HashMap;

use autd3_driver::{
    datagram::Datagram,
    derive::*,
    firmware::{
        cpu::TxDatagram,
        fpga::{
            GAIN_STM_BUF_SIZE_MAX, SAMPLING_FREQ_DIV_MAX, SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        },
        operation::{ControlPoint, FocusSTMOp, GainSTMChangeSegmentOp, GainSTMMode, GainSTMOp},
    },
    geometry::Vector3,
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
    let loop_behavior = LoopBehavior::Infinite;
    let segment = Segment::S0;
    let freq_div = rng.gen_range(
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
            ..=SAMPLING_FREQ_DIV_MAX,
    );
    let transition_mode = TransitionMode::SyncIdx;
    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseIntensityFull,
        freq_div,
        loop_behavior,
        segment,
        Some(transition_mode),
    );

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert!(cpu.fpga().is_stm_gain_mode(segment));
    assert_eq!(segment, cpu.fpga().current_stm_segment());
    assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
    assert_eq!(bufs.len(), cpu.fpga().stm_cycle(segment));
    assert_eq!(freq_div, cpu.fpga().stm_frequency_division(segment));
    assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives(segment, gain_idx)
            .into_iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(bufs[gain_idx][&0][i], drive);
            });
    });

    Ok(())
}

fn send_gain_stm_phase_full(n: usize) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(n, &geometry);
    let loop_behavior = LoopBehavior::Infinite;
    let segment = Segment::S1;
    let transition_mode = TransitionMode::Ext;
    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseFull,
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
        loop_behavior,
        segment,
        Some(transition_mode),
    );

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives(segment, gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(EmitIntensity::MAX, drive.intensity());
                assert_eq!(bufs[gain_idx][&0][i].phase(), drive.phase());
            });
        assert_eq!(segment, cpu.fpga().current_stm_segment());
        assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
        assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
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

    let loop_behavior = LoopBehavior::Infinite;
    let segment = Segment::S1;
    let transition_mode = TransitionMode::GPIO;
    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseHalf,
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
        loop_behavior,
        segment,
        Some(transition_mode),
    );

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives(segment, gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, &drive)| {
                assert_eq!(EmitIntensity::MAX, drive.intensity());
                assert_eq!(
                    bufs[gain_idx][&0][i].phase().value() >> 4,
                    drive.phase().value() >> 4
                );
            });
        assert_eq!(segment, cpu.fpga().current_stm_segment());
        assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
        assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
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

#[test]
fn send_gain_stm_invalid_segment_transition() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0: Gain
    {
        let buf: HashMap<usize, Vec<Drive>> = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::null()).collect()))
            .collect();
        let g = TestGain { buf: buf.clone() };

        let (mut op, _) = g.operation()?;

        send(&mut cpu, &mut op, &geometry, &mut tx)?;
    }

    // segment 1: FcousSTM
    {
        let freq_div = 0xFFFFFFFF;
        let foci = (0..2)
            .map(|_| ControlPoint::new(Vector3::zeros()))
            .collect();
        let loop_behaviour = LoopBehavior::Infinite;
        let segment = Segment::S1;
        let transition_mode = TransitionMode::Ext;
        let mut op = FocusSTMOp::new(
            foci,
            freq_div,
            loop_behaviour,
            segment,
            Some(transition_mode),
        );

        send(&mut cpu, &mut op, &geometry, &mut tx)?;
    }

    {
        let mut op = GainSTMChangeSegmentOp::new(Segment::S0, TransitionMode::default());
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );

        let mut op = GainSTMChangeSegmentOp::new(Segment::S1, TransitionMode::default());
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    Ok(())
}
