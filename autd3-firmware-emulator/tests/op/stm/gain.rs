use std::{collections::HashMap, num::NonZeroU16};

use autd3_core::{
    common::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    firmware::{
        Drive, GAIN_STM_BUF_SIZE_MAX, GPIOIn, Intensity, Phase, SamplingConfig, Segment,
        transition_mode::{Ext, GPIO, Immediate, Later, SyncIdx, TransitionMode},
    },
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    datagram::{
        ControlPoint, FixedCompletionSteps, FociSTM, GainSTM, GainSTMMode, GainSTMOption,
        PhaseCorrection, Silencer, SwapSegmentGainSTM, WithFiniteLoop, WithSegment,
    },
    error::AUTDDriverError,
    firmware::{
        cpu::check_firmware_err,
        operation::{OperationGenerator, OperationHandler},
    },
    geometry::{Geometry, Point3},
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

use rand::*;

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
                                intensity: Intensity(rng.random()),
                            })
                            .collect(),
                    )
                })
                .collect()
        })
        .collect()
}

#[test]
fn send_gain_stm_phase_intensity_full_infinite() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
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

    let bufs = gen_random_buf(GAIN_STM_BUF_SIZE_MAX, &geometry);
    let freq_div = rng.random_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as _..=u16::MAX,
    );
    let d = WithSegment {
        inner: GainSTM::new(
            bufs.iter()
                .map(|buf| TestGain { data: buf.clone() })
                .collect::<Vec<_>>(),
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            GainSTMOption::default(),
        ),
        segment: Segment::S0,
        transition_mode: Immediate,
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S0));
    assert_eq!(0xFFFF, cpu.fpga().stm_loop_count(Segment::S0));
    assert_eq!(bufs.len(), cpu.fpga().stm_cycle(Segment::S0));
    assert_eq!(freq_div, cpu.fpga().stm_freq_divide(Segment::S0));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives_at(Segment::S0, gain_idx)
            .into_iter()
            .enumerate()
            .for_each(|(i, drive)| {
                assert_eq!(bufs[gain_idx][&0][i].intensity, drive.intensity);
                assert_eq!(phase_corr[i] + bufs[gain_idx][&0][i].phase, drive.phase);
            });
    });

    Ok(())
}

#[test]
fn send_gain_stm_phase_intensity_full_unsafe() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
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

    let bufs = gen_random_buf(2, &geometry);
    let freq_div = rng.random_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as _..=u16::MAX,
    );
    let d = WithFiniteLoop {
        inner: GainSTM::new(
            bufs.iter()
                .map(|buf| TestGain { data: buf.clone() })
                .collect::<Vec<_>>(),
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            GainSTMOption::default(),
        ),
        loop_count: NonZeroU16::MIN,
        segment: Segment::S1,
        transition_mode: Later,
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(0, cpu.fpga().stm_loop_count(Segment::S1));
    assert_eq!(bufs.len(), cpu.fpga().stm_cycle(Segment::S1));
    assert_eq!(freq_div, cpu.fpga().stm_freq_divide(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
    (0..bufs.len()).for_each(|gain_idx| {
        cpu.fpga()
            .drives_at(Segment::S1, gain_idx)
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
#[case(2)]
#[case(3)]
#[case(GAIN_STM_BUF_SIZE_MAX)]
fn send_gain_stm_phase_full_unsafe(#[case] n: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    let bufs = gen_random_buf(n, &geometry);
    let segment = Segment::S1;
    let d = WithSegment {
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
        segment,
        transition_mode: Immediate,
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
                assert_eq!(Intensity::MAX, drive.intensity);
                assert_eq!(bufs[gain_idx][&0][i].phase, drive.phase);
            });
        assert_eq!(segment, cpu.fpga().req_stm_segment());
        assert_eq!(0xFFFF, cpu.fpga().stm_loop_count(segment));
        assert_eq!(Immediate.params(), cpu.fpga().stm_transition_mode());
    });

    Ok(())
}

#[rstest::rstest]
#[case(2)]
#[case(3)]
#[case(4)]
#[case(5)]
#[case(GAIN_STM_BUF_SIZE_MAX)]

fn send_gain_stm_phase_half_unsafe(#[case] n: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    [
        (Segment::S1, GPIOIn::I0),
        (Segment::S0, GPIOIn::I1),
        (Segment::S1, GPIOIn::I2),
        (Segment::S0, GPIOIn::I3),
    ]
    .into_iter()
    .try_for_each(
        |(segment, gpio)| -> Result<(), Box<dyn std::error::Error>> {
            let bufs = gen_random_buf(n, &geometry);

            let loop_count = NonZeroU16::MIN;
            let transition_mode = GPIO(gpio);
            let d = WithFiniteLoop {
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
                loop_count,
                segment,
                transition_mode,
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
                        assert_eq!(Intensity::MAX, drive.intensity);
                        assert_eq!(bufs[gain_idx][&0][i].phase.0 >> 4, drive.phase.0 >> 4);
                    });
                assert_eq!(segment, cpu.fpga().req_stm_segment());
                assert_eq!(loop_count.get() - 1, cpu.fpga().stm_loop_count(segment));
                assert_eq!(transition_mode.params(), cpu.fpga().stm_transition_mode());
            });
            Ok(())
        },
    )?;

    Ok(())
}

#[test]
fn change_gain_stm_segment_unsafe() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
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
        transition_mode: Later,
    };
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());

    let d = SwapSegmentGainSTM(Segment::S1, Immediate);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());

    Ok(())
}

#[test]
fn gain_stm_freq_div_too_small() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
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
            transition_mode: Later,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = Silencer {
            config: FixedCompletionSteps {
                intensity: NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT).unwrap(),
                phase: NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT * 2).unwrap(),
                strict: true,
            },
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegmentGainSTM(Segment::S1, Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_gain_stm_invalid_segment_transition() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
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

        let segment = Segment::S1;
        let transition_mode = Ext;
        let d = WithSegment {
            inner: FociSTM {
                foci: (0..2)
                    .map(|_| ControlPoint::from(Point3::origin()))
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            },
            segment,
            transition_mode,
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    {
        let d = SwapSegmentGainSTM(Segment::S0, Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegmentGainSTM(Segment::S1, Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_gain_stm_invalid_transition_mode() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    // segment 0 to 0
    {
        let d = WithFiniteLoop {
            inner: GainSTM {
                gains: gen_random_buf(2, &geometry)
                    .into_iter()
                    .map(|buf| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(NonZeroU16::MAX),
                option: GainSTMOption::default(),
            },
            segment: Segment::S0,
            loop_count: NonZeroU16::MIN,
            transition_mode: SyncIdx,
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
            transition_mode: Later,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegmentGainSTM(Segment::S1, SyncIdx);
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn invalid_gain_stm_mode() -> Result<(), Box<dyn std::error::Error>> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
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

    let mut generator =
        d.operation_generator(&geometry, &Environment::new(), &DeviceMask::AllEnabled)?;
    let mut op = geometry
        .iter()
        .map(|dev| generator.generate(dev))
        .collect::<Vec<_>>();
    OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)?;
    tx[0].payload_mut()[2] = 3;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDDriverError::InvalidGainSTMMode),
        check_firmware_err(cpu.rx().ack())
    );

    Ok(())
}
