use autd3_core::link::MsgId;
use std::{collections::HashMap, num::NonZeroU16};

use autd3_core::datagram::Datagram;
use autd3_driver::{
    datagram::{
        ControlPoint, FixedCompletionSteps, FociSTM, GainSTM, GainSTMOption, Silencer, SwapSegment,
        WithLoopBehavior, WithSegment,
    },
    error::AUTDDriverError,
    firmware::{
        cpu::{GainSTMMode, TxMessage},
        fpga::{
            Drive, EmitIntensity, GAIN_STM_BUF_SIZE_MAX, GPIOIn, LoopBehavior, Phase,
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT, SamplingConfig,
            Segment, TransitionMode,
        },
        operation::OperationHandler,
    },
    geometry::{Geometry, Point3},
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

use rand::*;
use zerocopy::FromZeros;

use super::super::gain::TestGain;

fn gen_random_buf(n: usize, geometry: &Geometry) -> Vec<HashMap<usize, Vec<Drive>>> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| {
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|_| Drive {
                                phase: Phase(rng.random()),
                                intensity: EmitIntensity(rng.random()),
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
    LoopBehavior::Infinite,
    Segment::S0,
    Some(TransitionMode::Immediate)
)]
#[case(2, LoopBehavior::ONCE, Segment::S1, None)]
fn send_gain_stm_phase_intensity_full_unsafe(
    #[case] n: usize,
    #[case] loop_behavior: LoopBehavior,
    #[case] segment: Segment,
    #[case] transition_mode: Option<TransitionMode>,
) -> anyhow::Result<()> {
    use autd3_driver::datagram::PhaseCorrection;

    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let phase_corr: Vec<_> = (0..geometry.num_transducers())
        .map(|_| Phase(rng.random()))
        .collect();
    {
        let d = PhaseCorrection::new(|_| |tr| phase_corr[tr.idx()]);
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    let bufs = gen_random_buf(n, &geometry);
    let freq_div = rng.random_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as _..=u16::MAX,
    );
    let d = WithLoopBehavior {
        inner: GainSTM::new(
            bufs.iter()
                .map(|buf| TestGain { data: buf.clone() })
                .collect::<Vec<_>>(),
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            GainSTMOption::default(),
        ),
        loop_behavior,
        segment,
        transition_mode,
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert!(cpu.fpga().is_stm_gain_mode(segment));
    assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(segment));
    assert_eq!(bufs.len(), cpu.fpga().stm_cycle(segment));
    assert_eq!(freq_div, cpu.fpga().stm_freq_divide(segment));
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
                assert_eq!(bufs[gain_idx][&0][i].intensity, drive.intensity);
                assert_eq!(phase_corr[i] + bufs[gain_idx][&0][i].phase, drive.phase);
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
fn send_gain_stm_phase_full_unsafe(#[case] n: usize) -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let bufs = gen_random_buf(n, &geometry);
    let loop_behavior = LoopBehavior::Infinite;
    let segment = Segment::S1;
    let transition_mode = TransitionMode::Ext;
    let d = WithLoopBehavior {
        inner: GainSTM {
            config: SamplingConfig::new(
                NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT))
                    .unwrap(),
            ),
            gains: bufs
                .iter()
                .map(|buf| TestGain { data: buf.clone() })
                .collect::<Vec<_>>(),
            option: GainSTMOption {
                mode: GainSTMMode::PhaseFull,
            },
        },
        loop_behavior,
        segment,
        transition_mode: Some(transition_mode),
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives_at(segment, gain_idx)
            .iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(EmitIntensity::MAX, drive.intensity);
                assert_eq!(bufs[gain_idx][&0][i].phase, drive.phase);
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

fn send_gain_stm_phase_half_unsafe(#[case] n: usize) -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    [
        (Segment::S1, GPIOIn::I0),
        (Segment::S0, GPIOIn::I1),
        (Segment::S1, GPIOIn::I2),
        (Segment::S0, GPIOIn::I3),
    ]
    .into_iter()
    .try_for_each(|(segment, gpio)| -> anyhow::Result<()> {
        let bufs = gen_random_buf(n, &geometry);

        let loop_behavior = LoopBehavior::ONCE;
        let transition_mode = TransitionMode::GPIO(gpio);
        let d = WithLoopBehavior {
            inner: GainSTM {
                config: SamplingConfig::new(
                    NonZeroU16::new(
                        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT),
                    )
                    .unwrap(),
                ),
                gains: bufs
                    .iter()
                    .map(|buf| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                option: GainSTMOption {
                    mode: GainSTMMode::PhaseHalf,
                },
            },
            loop_behavior,
            segment,
            transition_mode: Some(transition_mode),
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        (0..bufs.len()).for_each(|gain_idx| {
            cpu.fpga()
                .drives_at(segment, gain_idx)
                .iter()
                .enumerate()
                .for_each(|(i, &drive)| {
                    assert_eq!(EmitIntensity::MAX, drive.intensity);
                    assert_eq!(bufs[gain_idx][&0][i].phase.0 >> 4, drive.phase.0 >> 4);
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
fn change_gain_stm_segment_unsafe() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
    let d = WithSegment {
        inner: GainSTM {
            config: SamplingConfig::new(
                NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT))
                    .unwrap(),
            ),
            gains: gen_random_buf(2, &geometry)
                .into_iter()
                .map(|buf| TestGain { data: buf.clone() })
                .collect::<Vec<_>>(),
            option: GainSTMOption::default(),
        },
        segment: Segment::S1,
        transition_mode: None,
    };
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());

    let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());

    Ok(())
}

#[test]
fn gain_stm_freq_div_too_small() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let d = GainSTM {
            gains: gen_random_buf(2, &geometry)
                .into_iter()
                .map(|buf| TestGain { data: buf.clone() })
                .collect::<Vec<_>>(),
            config: SamplingConfig::FREQ_40K,
            option: GainSTMOption::default(),
        };

        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    {
        let d = TestGain {
            data: geometry
                .iter()
                .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::NULL).collect()))
                .collect(),
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = Silencer::<FixedCompletionSteps>::default();
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = WithSegment {
            inner: GainSTM {
                gains: gen_random_buf(2, &geometry)
                    .into_iter()
                    .map(|buf| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(
                    NonZeroU16::new(
                        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT),
                    )
                    .unwrap(),
                ),
                option: GainSTMOption::default(),
            },
            segment: Segment::S1,
            transition_mode: None,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = Silencer {
            config: FixedCompletionSteps {
                intensity: NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT).unwrap(),
                phase: NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT * 2).unwrap(),
                strict_mode: true,
            },
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_gain_stm_invalid_segment_transition() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    // segment 0: Gain
    {
        let buf: HashMap<usize, Vec<Drive>> = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::NULL).collect()))
            .collect();
        let d = TestGain { data: buf.clone() };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    // segment 1: FocusSTM
    {
        let freq_div = 0xFFFF;

        let loop_behavior = LoopBehavior::Infinite;
        let segment = Segment::S1;
        let transition_mode = TransitionMode::Ext;
        let d = WithLoopBehavior {
            inner: FociSTM {
                foci: (0..2)
                    .map(|_| ControlPoint::from(Point3::origin()))
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            },
            segment,
            transition_mode: Some(transition_mode),
            loop_behavior,
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    {
        let d = SwapSegment::GainSTM(Segment::S0, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_gain_stm_invalid_transition_mode() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    // segment 0 to 0
    {
        let d = WithSegment {
            inner: GainSTM {
                gains: gen_random_buf(2, &geometry)
                    .into_iter()
                    .map(|buf| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(NonZeroU16::MAX),
                option: GainSTMOption::default(),
            },
            segment: Segment::S0,
            transition_mode: Some(TransitionMode::SyncIdx),
        };
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    // segment 0 to 1 immediate
    {
        let d = WithLoopBehavior {
            inner: GainSTM {
                gains: gen_random_buf(2, &geometry)
                    .into_iter()
                    .map(|buf| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(NonZeroU16::MAX),
                option: GainSTMOption::default(),
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
            loop_behavior: LoopBehavior::ONCE,
        };
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    // Infinite but SyncIdx
    {
        let d = WithSegment {
            inner: GainSTM {
                gains: gen_random_buf(2, &geometry)
                    .into_iter()
                    .map(|buf| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(NonZeroU16::MAX),
                option: GainSTMOption::default(),
            },

            segment: Segment::S1,
            transition_mode: None,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegment::GainSTM(Segment::S1, TransitionMode::SyncIdx);
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn invalid_gain_stm_mode() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut sent_flags = vec![false; 1];
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let msg_id = MsgId::new(0);

    let bufs = gen_random_buf(2, &geometry);
    let d = GainSTM {
        gains: bufs
            .iter()
            .map(|buf| TestGain { data: buf.clone() })
            .collect::<Vec<_>>(),
        config: SamplingConfig::new(NonZeroU16::MAX),
        option: GainSTMOption::default(),
    };

    let generator = d.operation_generator(&mut geometry)?;
    let mut op = OperationHandler::generate(generator, &geometry);
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut sent_flags, &mut tx, false)?;
    tx[0].payload_mut()[2] = 3;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDDriverError::InvalidGainSTMMode),
        autd3_driver::firmware::cpu::check_firmware_err(&cpu.rx())
    );

    Ok(())
}
