use std::collections::HashMap;

use autd3_driver::{
    datagram::{ConfigureSilencer, ControlPoint, Datagram},
    derive::*,
    firmware::{
        cpu::{GainSTMMode, TxDatagram},
        fpga::{
            GPIOIn, STMSamplingConfig, GAIN_STM_BUF_SIZE_MAX, SAMPLING_FREQ_DIV_MAX,
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
        },
        operation::{
            FocusSTMOp, GainSTMOp, GainSTMSwapSegmentOp, OperationHandler, SwapSegmentOperation,
        },
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

    {
        let bufs = gen_random_buf(GAIN_STM_BUF_SIZE_MAX, &geometry);
        let loop_behavior = LoopBehavior::infinite();
        let segment = Segment::S0;
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let transition_mode = TransitionMode::Immidiate;
        let mut op = GainSTMOp::new(
            bufs.iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(freq_div)),
            loop_behavior,
            segment,
            Some(transition_mode),
        );

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        assert!(cpu.fpga().is_stm_gain_mode(segment));
        assert_eq!(segment, cpu.fpga().current_stm_segment());
        assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
        assert_eq!(bufs.len(), cpu.fpga().stm_cycle(segment));
        assert_eq!(freq_div, cpu.fpga().stm_freq_division(segment));
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
    }

    {
        let bufs = gen_random_buf(2, &geometry);
        let loop_behavior = LoopBehavior::once();
        let segment = Segment::S1;
        let freq_div = SAMPLING_FREQ_DIV_MAX;
        let mut op = GainSTMOp::new(
            bufs.iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(freq_div)),
            loop_behavior,
            segment,
            None,
        );

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        assert!(cpu.fpga().is_stm_gain_mode(segment));
        assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());
        assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
        assert_eq!(bufs.len(), cpu.fpga().stm_cycle(segment));
        assert_eq!(freq_div, cpu.fpga().stm_freq_division(segment));
        assert_eq!(TransitionMode::Immidiate, cpu.fpga().stm_transition_mode());
        (0..bufs.len()).for_each(|gain_idx| {
            cpu.fpga()
                .drives(segment, gain_idx)
                .into_iter()
                .enumerate()
                .for_each(|(i, drive)| {
                    assert_eq!(bufs[gain_idx][&0][i], drive);
                });
        });
    }

    {
        let mut op = GainSTMSwapSegmentOp::new(Segment::S1, TransitionMode::SyncIdx);

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        assert_eq!(Segment::S1, cpu.fpga().current_stm_segment());
        assert_eq!(TransitionMode::SyncIdx, cpu.fpga().stm_transition_mode());
    }

    Ok(())
}

fn send_gain_stm_phase_full(n: usize) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(n, &geometry);
    let loop_behavior = LoopBehavior::infinite();
    let segment = Segment::S1;
    let transition_mode = TransitionMode::Ext;
    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseFull,
        STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
        )),
        loop_behavior,
        segment,
        Some(transition_mode),
    );

    assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

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

    [
        (Segment::S1, GPIOIn::I0),
        (Segment::S0, GPIOIn::I1),
        (Segment::S1, GPIOIn::I2),
        (Segment::S0, GPIOIn::I3),
    ]
    .into_iter()
    .for_each(|(segment, gpio)| {
        let bufs = gen_random_buf(n, &geometry);

        let loop_behavior = LoopBehavior::once();
        let transition_mode = TransitionMode::GPIO(gpio);
        let mut op = GainSTMOp::new(
            bufs.iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseHalf,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(
                SAMPLING_FREQ_DIV_MIN
                    * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
            )),
            loop_behavior,
            segment,
            Some(transition_mode),
        );

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

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
fn change_gain_stm_segment() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());
    let mut op = GainSTMOp::new(
        gen_random_buf(GAIN_STM_BUF_SIZE_MAX, &geometry)
            .iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseIntensityFull,
        STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX)),
        LoopBehavior::infinite(),
        Segment::S1,
        None,
    );
    assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());

    let mut op = GainSTMSwapSegmentOp::new(Segment::S1, TransitionMode::Immidiate);
    assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().current_stm_segment());

    Ok(())
}

#[test]
fn gain_stm_freq_div_too_small() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let mut op = GainSTMOp::new(
            gen_random_buf(2, &geometry)
                .iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MIN)),
            LoopBehavior::infinite(),
            Segment::S0,
            Some(TransitionMode::Immidiate),
        );

        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    {
        let g = TestGain {
            buf: geometry
                .iter()
                .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::null()).collect()))
                .collect(),
        };
        let (mut op, _) = g.operation_with_segment(Segment::S0, true);
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let (mut op, _) = ConfigureSilencer::fixed_completion_steps(
            SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        )?
        .operation();
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let mut op = GainSTMOp::new(
            gen_random_buf(2, &geometry)
                .iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(
                SAMPLING_FREQ_DIV_MIN
                    * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
            )),
            LoopBehavior::infinite(),
            Segment::S1,
            None,
        );
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let (mut op, _) = ConfigureSilencer::fixed_completion_steps(
            SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT * 2,
        )?
        .operation();
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let mut op = GainSTMSwapSegmentOp::new(Segment::S1, TransitionMode::Immidiate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

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

        let (mut op, _) = g.operation_with_segment(Segment::S0, true);

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));
    }

    // segment 1: FcousSTM
    {
        let freq_div = 0xFFFFFFFF;
        let foci = (0..2)
            .map(|_| ControlPoint::new(Vector3::zeros()))
            .collect();
        let loop_behaviour = LoopBehavior::infinite();
        let segment = Segment::S1;
        let transition_mode = TransitionMode::Ext;
        let mut op = FocusSTMOp::new(
            foci,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(freq_div)),
            loop_behaviour,
            segment,
            Some(transition_mode),
        );

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));
    }

    {
        let mut op = GainSTMSwapSegmentOp::new(Segment::S0, TransitionMode::Immidiate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );

        let mut op = GainSTMSwapSegmentOp::new(Segment::S1, TransitionMode::Immidiate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_gain_stm_invalid_transition_mode() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0 to 0
    {
        let mut op = GainSTMOp::new(
            gen_random_buf(2, &geometry)
                .iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX)),
            LoopBehavior::infinite(),
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    // segment 0 to 1 immidiate
    {
        let mut op = GainSTMOp::new(
            gen_random_buf(2, &geometry)
                .iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX)),
            LoopBehavior::once(),
            Segment::S1,
            Some(TransitionMode::Immidiate),
        );
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    // Infinite but SyncIdx
    {
        let mut op = GainSTMOp::new(
            gen_random_buf(2, &geometry)
                .iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX)),
            LoopBehavior::infinite(),
            Segment::S1,
            None,
        );
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let mut op = GainSTMSwapSegmentOp::new(Segment::S1, TransitionMode::SyncIdx);
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn invalid_gain_stm_mode() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(2, &geometry);
    let mut op = GainSTMOp::new(
        bufs.iter()
            .map(|buf| TestGain { buf: buf.clone() })
            .collect(),
        GainSTMMode::PhaseIntensityFull,
        STMSamplingConfig::SamplingConfig(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX)),
        LoopBehavior::infinite(),
        Segment::S0,
        Some(TransitionMode::Immidiate),
    );
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry)?;
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx)?;
    tx[0].payload[2] = 3;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::InvalidGainSTMMode),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
