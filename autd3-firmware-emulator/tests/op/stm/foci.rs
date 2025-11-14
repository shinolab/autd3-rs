use std::{collections::HashMap, num::NonZeroU16, time::Duration};

use autd3_core::{
    common::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
    environment::Environment,
    firmware::{
        Drive, FOCI_STM_BUF_SIZE_MAX, FOCI_STM_FIXED_NUM_UNIT, Intensity, Phase, SamplingConfig,
        Segment,
        transition_mode::{Immediate, Later, SyncIdx, TransitionMode},
    },
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    common::{METER, mm},
    datagram::{
        ControlPoint, ControlPoints, FixedCompletionSteps, FociSTM, GainSTM, GainSTMOption,
        PhaseCorrection, Silencer, SwapSegmentFociSTM, WithFiniteLoop, WithSegment,
    },
    error::AUTDDriverError,
    ethercat::DcSysTime,
    geometry::Point3,
};
use autd3_firmware_emulator::{CPUEmulator, cpu::params::SYS_TIME_TRANSITION_MARGIN};

use crate::{create_geometry, op::gain::TestGain, send};
use rand::*;

pub fn gen_random_foci<const N: usize>(num: usize) -> Vec<ControlPoints<N>> {
    let mut rng = rand::rng();
    (0..num)
        .map(|_| {
            ControlPoints::new(
                [0; N].map(|_| {
                    ControlPoint::new(
                        Point3::new(
                            rng.random_range(-100.0 * mm..100.0 * mm),
                            rng.random_range(-100.0 * mm..100.0 * mm),
                            rng.random_range(-100.0 * mm..100.0 * mm),
                        ),
                        Phase(rng.random()),
                    )
                }),
                Intensity(rng.random()),
            )
        })
        .collect()
}

#[test]
fn send_foci_stm_infinite() -> Result<(), Box<dyn std::error::Error>> {
    let sin_table = include_bytes!("sin.dat");
    let atan_table = include_bytes!("atan.dat");

    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let env = Environment::new();
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

    let freq_div = rng.random_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as _..=u16::MAX,
    );
    let foci = gen_random_foci::<1>(FOCI_STM_BUF_SIZE_MAX);

    let stm = WithSegment {
        inner: FociSTM::new(
            foci.as_ref(),
            SamplingConfig::new(unsafe { NonZeroU16::new_unchecked(freq_div) }),
        ),
        segment: Segment::S0,
        transition_mode: Immediate,
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
    );

    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S0));
    assert_eq!(0xFFFF, cpu.fpga().stm_loop_count(Segment::S0));
    assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S0));
    assert_eq!(freq_div, cpu.fpga().stm_freq_divide(Segment::S0));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
    assert_eq!(
        (env.sound_speed / METER * 64.0).round() as u16,
        cpu.fpga().sound_speed(Segment::S0)
    );
    foci.iter().enumerate().for_each(|(focus_idx, focus)| {
        let drives = cpu.fpga().drives_at(Segment::S0, focus_idx);
        assert_eq!(cpu.num_transducers(), drives.len());
        drives.iter().enumerate().for_each(|(tr_idx, &drive)| {
            let tr = cpu.fpga().local_tr_pos_at(tr_idx);
            let tx = ((tr >> 16) & 0xFFFF) as i32;
            let ty = (tr & 0xFFFF) as i16 as i32;
            let tz = 0;
            let fx = (focus[0].point.x / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
            let fy = (focus[0].point.y / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
            let fz = (focus[0].point.z / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
            let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).isqrt() as u32;
            let q = (d << 14) / cpu.fpga().sound_speed(Segment::S0) as u32;
            let sin = (sin_table[q as usize % 256] >> 1) as usize;
            let cos = (sin_table[(q as usize + 64) % 256] >> 1) as usize;
            let p = atan_table[(sin << 7) | cos];
            assert_eq!(Phase(p) + phase_corr[tr_idx], drive.phase);
            assert_eq!(focus.intensity, drive.intensity);
        })
    });

    Ok(())
}

#[test]
fn send_foci_stm_finite() -> Result<(), Box<dyn std::error::Error>> {
    let sin_table = include_bytes!("sin.dat");
    let atan_table = include_bytes!("atan.dat");

    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let env = Environment::new();
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

    let freq_div = rng.random_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as _..=u16::MAX,
    );
    let foci = gen_random_foci::<1>(2);

    let stm = WithFiniteLoop {
        inner: FociSTM::new(
            foci.as_ref(),
            SamplingConfig::new(unsafe { NonZeroU16::new_unchecked(freq_div) }),
        ),
        segment: Segment::S1,
        loop_count: NonZeroU16::MIN,
        transition_mode: SyncIdx,
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
    );

    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(0, cpu.fpga().stm_loop_count(Segment::S1));
    assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S1));
    assert_eq!(freq_div, cpu.fpga().stm_freq_divide(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());
    assert_eq!(SyncIdx.params(), cpu.fpga().stm_transition_mode());
    assert_eq!(
        (env.sound_speed / METER * 64.0).round() as u16,
        cpu.fpga().sound_speed(Segment::S1)
    );
    foci.iter().enumerate().for_each(|(focus_idx, focus)| {
        let drives = cpu.fpga().drives_at(Segment::S1, focus_idx);
        assert_eq!(cpu.num_transducers(), drives.len());
        drives.iter().enumerate().for_each(|(tr_idx, &drive)| {
            let tr = cpu.fpga().local_tr_pos_at(tr_idx);
            let tx = ((tr >> 16) & 0xFFFF) as i32;
            let ty = (tr & 0xFFFF) as i16 as i32;
            let tz = 0;
            let fx = (focus[0].point.x / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
            let fy = (focus[0].point.y / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
            let fz = (focus[0].point.z / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
            let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).isqrt() as u32;
            let q = (d << 14) / cpu.fpga().sound_speed(Segment::S1) as u32;
            let sin = (sin_table[q as usize % 256] >> 1) as usize;
            let cos = (sin_table[(q as usize + 64) % 256] >> 1) as usize;
            let p = atan_table[(sin << 7) | cos];
            assert_eq!(Phase(p) + phase_corr[tr_idx], drive.phase);
            assert_eq!(focus.intensity, drive.intensity);
        })
    });

    Ok(())
}

#[test]
fn change_foci_stm_segment() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());

    let stm = WithSegment {
        inner: FociSTM {
            foci: gen_random_foci::<1>(2),
            config: SamplingConfig::new(NonZeroU16::MAX),
        },
        segment: Segment::S1,
        transition_mode: Later,
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
    );
    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());

    let d = SwapSegmentFociSTM(Segment::S1, Immediate);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());

    Ok(())
}

#[test]
fn foci_stm_freq_div_too_small() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let stm = FociSTM {
            foci: gen_random_foci::<1>(2),
            config: SamplingConfig::FREQ_4K,
        };

        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
        );
    }

    {
        let g = TestGain {
            data: geometry
                .iter()
                .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::NULL).collect()))
                .collect(),
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, g, &mut geometry, &mut tx)
        );

        let d = Silencer::<FixedCompletionSteps>::default();
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let stm = WithSegment {
            inner: FociSTM {
                foci: gen_random_foci::<1>(2),
                config: SamplingConfig::new(unsafe {
                    NonZeroU16::new_unchecked(
                        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT),
                    )
                }),
            },
            segment: Segment::S1,
            transition_mode: Later,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
        );

        let d = Silencer {
            config: FixedCompletionSteps {
                intensity: unsafe { NonZeroU16::new_unchecked(SILENCER_STEPS_INTENSITY_DEFAULT) },
                phase: unsafe { NonZeroU16::new_unchecked(SILENCER_STEPS_PHASE_DEFAULT * 2) },
                strict: true,
            },
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegmentFociSTM(Segment::S1, Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_foci_stm_invalid_segment_transition() -> Result<(), Box<dyn std::error::Error>> {
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
        let g = TestGain { data: buf.clone() };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, g, &mut geometry, &mut tx)
        );
    }

    // segment 1: GainSTM
    {
        let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..2)
            .map(|_| {
                geometry
                    .iter()
                    .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::NULL).collect()))
                    .collect()
            })
            .collect();
        let stm = WithSegment {
            inner: GainSTM {
                gains: bufs
                    .iter()
                    .map(|buf| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(NonZeroU16::MAX),
                option: GainSTMOption::default(),
            },
            segment: Segment::S1,
            transition_mode: Immediate,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
        );
    }

    {
        let d = SwapSegmentFociSTM(Segment::S0, Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegmentFociSTM(Segment::S1, Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_foci_stm_invalid_transition_mode() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    // segment 0 to 0
    {
        let stm = WithFiniteLoop {
            inner: FociSTM {
                foci: gen_random_foci::<1>(2),
                config: SamplingConfig::new(NonZeroU16::MAX),
            },
            segment: Segment::S0,
            loop_count: NonZeroU16::MIN,
            transition_mode: SyncIdx,
        };
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
        );
    }

    // Infinite but SyncIdx
    {
        let stm = WithSegment {
            inner: FociSTM {
                foci: gen_random_foci::<1>(2),
                config: SamplingConfig::new(NonZeroU16::MAX),
            },
            segment: Segment::S1,
            transition_mode: Later,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
        );

        let d = SwapSegmentFociSTM(Segment::S1, SyncIdx);
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[rstest::rstest]
#[case(Ok(()), DcSysTime::ZERO, DcSysTime::ZERO + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
#[case(Err(AUTDDriverError::MissTransitionTime), DcSysTime::ZERO, DcSysTime::ZERO + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN)-autd3_driver::ethercat::EC_CYCLE_TIME_BASE)]
#[case(Err(AUTDDriverError::MissTransitionTime), DcSysTime::ZERO + Duration::from_nanos(1), DcSysTime::ZERO + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
fn miss_transition_time(
    #[case] expect: Result<(), AUTDDriverError>,
    #[case] systime: DcSysTime,
    #[case] transition_time: DcSysTime,
) -> Result<(), Box<dyn std::error::Error>> {
    use autd3_core::firmware::transition_mode;

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    let transition_mode = transition_mode::SysTime(transition_time);

    let stm = WithFiniteLoop {
        inner: FociSTM {
            foci: gen_random_foci::<1>(2),
            config: SamplingConfig::new(NonZeroU16::MAX),
        },
        segment: Segment::S1,
        transition_mode,
        loop_count: NonZeroU16::MIN,
    };

    cpu.update_with_sys_time(systime);
    assert_eq!(
        expect,
        send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
    );
    if expect.is_ok() {
        assert_eq!(transition_mode.params(), cpu.fpga().stm_transition_mode());
    }

    Ok(())
}

fn send_foci_stm_n<const N: usize>() -> Result<(), Box<dyn std::error::Error>> {
    let sin_table = include_bytes!("sin.dat");
    let atan_table = include_bytes!("atan.dat");

    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let env = Environment::new();
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let freq_div = rng.random_range(
            SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as _..=u16::MAX,
        );
        let foci = gen_random_foci::<N>(1000);
        let segment = Segment::S0;

        let stm = WithSegment {
            inner: FociSTM {
                foci: foci.as_ref(),
                config: SamplingConfig::new(unsafe { NonZeroU16::new_unchecked(freq_div) }),
            },
            segment,
            transition_mode: Immediate,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, stm, &mut geometry, &mut tx)
        );

        assert!(!cpu.fpga().is_stm_gain_mode(Segment::S0));
        assert_eq!(segment, cpu.fpga().req_stm_segment());
        assert_eq!(0xFFFF, cpu.fpga().stm_loop_count(Segment::S0));
        assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S0));
        assert_eq!(freq_div, cpu.fpga().stm_freq_divide(Segment::S0));
        assert_eq!(Immediate.params(), cpu.fpga().stm_transition_mode());
        assert_eq!(
            (env.sound_speed / METER * 64.0).round() as u16,
            cpu.fpga().sound_speed(Segment::S0)
        );
        foci.iter().enumerate().for_each(|(focus_idx, focus)| {
            cpu.fpga()
                .drives_at(Segment::S0, focus_idx)
                .iter()
                .enumerate()
                .for_each(|(tr_idx, &drive)| {
                    let tr = cpu.fpga().local_tr_pos_at(tr_idx);
                    let tx = ((tr >> 16) & 0xFFFF) as i32;
                    let ty = (tr & 0xFFFF) as i16 as i32;
                    let tz = 0;
                    let base_offset = focus[0].phase_offset;
                    let (sin, cos) = focus.into_iter().fold((0, 0), |acc, f| {
                        let fx = (f.point.x / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let fy = (f.point.y / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let fz = (f.point.z / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let d =
                            ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).isqrt() as u32;
                        let q = (d << 14) / cpu.fpga().sound_speed(Segment::S0) as u32
                            + (f.phase_offset - base_offset).0 as u32;
                        let sin = sin_table[q as usize % 256] as usize;
                        let cos = sin_table[(q as usize + 64) % 256] as usize;
                        (acc.0 + sin, acc.1 + cos)
                    });
                    let (sin, cos) = ((sin / N) >> 1, (cos / N) >> 1);
                    let p = atan_table[(sin << 7) | cos];
                    assert_eq!(Phase(p), drive.phase);
                    assert_eq!(focus.intensity, drive.intensity);
                })
        });
    }

    Ok(())
}

#[test]
fn send_foci_stm_2() -> Result<(), Box<dyn std::error::Error>> {
    send_foci_stm_n::<2>()
}

#[test]
fn send_foci_stm_3() -> Result<(), Box<dyn std::error::Error>> {
    send_foci_stm_n::<3>()
}

#[test]
fn send_foci_stm_4() -> Result<(), Box<dyn std::error::Error>> {
    send_foci_stm_n::<4>()
}

#[test]
fn send_foci_stm_5() -> Result<(), Box<dyn std::error::Error>> {
    send_foci_stm_n::<5>()
}

#[test]
fn send_foci_stm_6() -> Result<(), Box<dyn std::error::Error>> {
    send_foci_stm_n::<6>()
}

#[test]
fn send_foci_stm_7() -> Result<(), Box<dyn std::error::Error>> {
    send_foci_stm_n::<7>()
}

#[test]
fn send_foci_stm_8() -> Result<(), Box<dyn std::error::Error>> {
    send_foci_stm_n::<8>()
}
