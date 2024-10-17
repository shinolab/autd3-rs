use std::collections::HashMap;

use autd3_driver::{
    datagram::{
        FixedCompletionTime, FociSTM, GainSTM, IntoDatagramWithSegment,
        IntoDatagramWithSegmentTransition, Silencer, SwapSegment,
    },
    defined::ControlPoint,
    derive::*,
    firmware::{
        cpu::{GainSTMMode, TxDatagram},
        fpga::{
            Drive, EmitIntensity, GPIOIn, Phase, GAIN_STM_BUF_SIZE_MAX,
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
        },
        operation::OperationHandler,
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

#[rstest::rstest]
#[test]
#[cfg_attr(miri, ignore)]
#[case(
    GAIN_STM_BUF_SIZE_MAX,
    LoopBehavior::infinite(),
    Segment::S0,
    Some(TransitionMode::Immediate)
)]
#[case(2, LoopBehavior::once(), Segment::S1, None)]
fn send_gain_stm_phase_intensity_full(
    #[case] n: usize,
    #[case] loop_behavior: LoopBehavior,
    #[case] segment: Segment,
    #[case] transition_mode: Option<TransitionMode>,
) -> anyhow::Result<()> {
    use autd3_driver::datagram::PhaseCorrection;

    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let phase_corr: Vec<_> = (0..geometry.num_transducers())
        .map(|_| Phase::new(rng.gen()))
        .collect();
    {
        let d = PhaseCorrection::new(|_| |tr| phase_corr[tr.idx()]);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let bufs = gen_random_buf(n, &geometry);
    let freq_div = rng.gen_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as _..=u16::MAX,
    );
    let d = GainSTM::new(
        SamplingConfig::new(freq_div).unwrap(),
        bufs.iter().map(|buf| TestGain { data: buf.clone() }),
    )?
    .with_loop_behavior(loop_behavior)
    .with_segment(segment, transition_mode);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert!(cpu.fpga().is_stm_gain_mode(segment));
    assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
    assert_eq!(bufs.len(), cpu.fpga().stm_cycle(segment));
    assert_eq!(freq_div, cpu.fpga().stm_freq_division(segment));
    if let Some(transition_mode) = transition_mode {
        assert_eq!(segment, cpu.fpga().req_stm_segment());
        assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
    } else {
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
    }
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives_at(segment, gain_idx)
            .into_iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(bufs[gain_idx][&0][i].intensity(), drive.intensity());
                assert_eq!(phase_corr[i] + bufs[gain_idx][&0][i].phase(), drive.phase());
            });
    });

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(2)]
#[cfg_attr(miri, ignore)]
#[case(3)]
#[cfg_attr(miri, ignore)]
#[case(GAIN_STM_BUF_SIZE_MAX)]
fn send_gain_stm_phase_full(#[case] n: usize) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(n, &geometry);
    let loop_behavior = LoopBehavior::infinite();
    let segment = Segment::S1;
    let transition_mode = TransitionMode::Ext;
    let d = GainSTM::new(
        SamplingConfig::new(
            SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u16,
        )
        .unwrap(),
        bufs.iter().map(|buf| TestGain { data: buf.clone() }),
    )?
    .with_mode(GainSTMMode::PhaseFull)
    .with_loop_behavior(loop_behavior)
    .with_segment(segment, Some(transition_mode));

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives_at(segment, gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(EmitIntensity::MAX, drive.intensity());
                assert_eq!(bufs[gain_idx][&0][i].phase(), drive.phase());
            });
        assert_eq!(segment, cpu.fpga().req_stm_segment());
        assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
        assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
    });

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(2)]
#[cfg_attr(miri, ignore)]
#[case(3)]
#[cfg_attr(miri, ignore)]
#[case(4)]
#[cfg_attr(miri, ignore)]
#[case(5)]
#[cfg_attr(miri, ignore)]
#[case(GAIN_STM_BUF_SIZE_MAX)]

fn send_gain_stm_phase_half(#[case] n: usize) -> anyhow::Result<()> {
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
    .try_for_each(|(segment, gpio)| -> anyhow::Result<()> {
        let bufs = gen_random_buf(n, &geometry);

        let loop_behavior = LoopBehavior::once();
        let transition_mode = TransitionMode::GPIO(gpio);
        let d = GainSTM::new(
            SamplingConfig::new(
                SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u16,
            )
            .unwrap(),
            bufs.iter().map(|buf| TestGain { data: buf.clone() }),
        )?
        .with_mode(GainSTMMode::PhaseHalf)
        .with_loop_behavior(loop_behavior)
        .with_segment(segment, Some(transition_mode));

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        (0..bufs.len()).for_each(|gain_idx| {
            cpu.fpga()
                .drives_at(segment, gain_idx)
                .iter()
                .enumerate()
                .for_each(|(i, &drive)| {
                    assert_eq!(EmitIntensity::MAX, drive.intensity());
                    assert_eq!(
                        bufs[gain_idx][&0][i].phase().value() >> 4,
                        drive.phase().value() >> 4
                    );
                });
            assert_eq!(segment, cpu.fpga().req_stm_segment());
            assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
            assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
        });
        Ok(())
    })?;

    Ok(())
}

#[test]
fn change_gain_stm_segment() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
    let d = GainSTM::new(
        SamplingConfig::FREQ_MIN,
        gen_random_buf(2, &geometry)
            .into_iter()
            .map(|buf| TestGain { data: buf.clone() }),
    )?
    .with_segment(Segment::S1, None);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());

    let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn gain_stm_freq_div_too_small() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let d = GainSTM::new(
            SamplingConfig::FREQ_40K,
            gen_random_buf(2, &geometry)
                .into_iter()
                .map(|buf| TestGain { data: buf.clone() }),
        )?
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));

        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    {
        let d = TestGain {
            data: geometry
                .iter()
                .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::null()).collect()))
                .collect(),
        }
        .with_segment(Segment::S0, true);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = Silencer::<FixedCompletionTime>::default();
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = GainSTM::new(
            SamplingConfig::new(
                SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u16,
            )
            .unwrap(),
            gen_random_buf(2, &geometry)
                .into_iter()
                .map(|buf| TestGain { data: buf.clone() }),
        )?
        .with_segment(Segment::S1, None);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = Silencer::new(FixedCompletionTime {
            intensity: Silencer::DEFAULT_COMPLETION_TIME_INTENSITY,
            phase: Silencer::DEFAULT_COMPLETION_TIME_PHASE * 2,
        });
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
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
        let d = TestGain { data: buf.clone() }.with_segment(Segment::S0, true);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    // segment 1: FcousSTM
    {
        let freq_div = 0xFFFF;

        let loop_behaviour = LoopBehavior::infinite();
        let segment = Segment::S1;
        let transition_mode = TransitionMode::Ext;
        let d = FociSTM::new(
            SamplingConfig::new(freq_div).unwrap(),
            (0..2).map(|_| ControlPoint::new(Vector3::zeros())),
        )?
        .with_loop_behavior(loop_behaviour)
        .with_segment(segment, Some(transition_mode));

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    {
        let d = SwapSegment::GainSTM(Segment::S0, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );

        let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_gain_stm_invalid_transition_mode() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0 to 0
    {
        let d = GainSTM::new(
            SamplingConfig::FREQ_MIN,
            gen_random_buf(2, &geometry)
                .into_iter()
                .map(|buf| TestGain { data: buf.clone() }),
        )?
        .with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    // segment 0 to 1 immidiate
    {
        let d = GainSTM::new(
            SamplingConfig::FREQ_MIN,
            gen_random_buf(2, &geometry)
                .into_iter()
                .map(|buf| TestGain { data: buf.clone() }),
        )?
        .with_loop_behavior(LoopBehavior::once())
        .with_segment(Segment::S1, Some(TransitionMode::Immediate));
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    // Infinite but SyncIdx
    {
        let d = GainSTM::new(
            SamplingConfig::FREQ_MIN,
            gen_random_buf(2, &geometry)
                .into_iter()
                .map(|buf| TestGain { data: buf.clone() }),
        )?
        .with_segment(Segment::S1, None);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::SyncIdx);
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn invalid_gain_stm_mode() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let bufs = gen_random_buf(2, &geometry);
    let d = GainSTM::new(
        SamplingConfig::FREQ_MIN,
        bufs.iter().map(|buf| TestGain { data: buf.clone() }),
    )?
    .with_segment(Segment::S0, Some(TransitionMode::Immediate));

    let generator = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, false)?;
    tx[0].payload[2] = 3;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::InvalidGainSTMMode),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
