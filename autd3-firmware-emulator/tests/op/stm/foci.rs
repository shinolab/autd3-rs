use std::{collections::HashMap, num::NonZeroU16, time::Duration};

use autd3_driver::{
    datagram::{FociSTM, GainSTM, IntoDatagramWithSegmentTransition, Silencer, SwapSegment},
    defined::{mm, ControlPoint, ControlPoints, METER},
    derive::{Drive, LoopBehavior, Phase, SamplingConfig, Segment},
    error::AUTDInternalError,
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::{
        cpu::TxDatagram,
        fpga::{
            TransitionMode, FOCI_STM_BUF_SIZE_MAX, FOCI_STM_FIXED_NUM_UNIT,
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
        },
    },
    geometry::Vector3,
};
use autd3_firmware_emulator::{cpu::params::SYS_TIME_TRANSITION_MARGIN, CPUEmulator};

use num_integer::Roots;
use rand::*;
use time::OffsetDateTime;

use crate::{create_geometry, op::gain::TestGain, send};

pub fn gen_random_foci<const N: usize>(num: usize) -> Vec<ControlPoints<N>> {
    let mut rng = rand::thread_rng();
    (0..num)
        .map(|_| {
            ControlPoints::new([0; N].map(|_| {
                ControlPoint::new(Vector3::new(
                    rng.gen_range(-100.0 * mm..100.0 * mm),
                    rng.gen_range(-100.0 * mm..100.0 * mm),
                    rng.gen_range(-100.0 * mm..100.0 * mm),
                ))
                .with_offset(rng.gen::<u8>())
            }))
            .with_intensity(rng.gen::<u8>())
        })
        .collect()
}

#[test]
fn test_send_foci_stm() {
    temp_env::with_var("AUTD3_ULTRASOUND_FREQ", Some("40000"), || {
        let sin_table = include_bytes!("sin.dat");
        let atan_table = include_bytes!("atan.dat");

        let mut rng = rand::thread_rng();

        let mut geometry = create_geometry(1);
        geometry.set_sound_speed(400e3);
        let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
        let mut tx = TxDatagram::new(geometry.num_devices());

        {
            let freq_div = rng.gen_range(
                SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT)..=0xFFFF,
            );
            let foci = gen_random_foci::<1>(FOCI_STM_BUF_SIZE_MAX);
            let loop_behavior = LoopBehavior::infinite();
            let segment = Segment::S0;
            let transition_mode = TransitionMode::Immediate;

            let stm = FociSTM::from_sampling_config(
                SamplingConfig::Division(NonZeroU16::new(freq_div).unwrap()),
                foci.clone(),
            )
            .with_loop_behavior(loop_behavior)
            .with_segment(segment, Some(transition_mode));

            assert_eq!(Ok(()), send(&mut cpu, stm, &geometry, &mut tx));

            assert!(!cpu.fpga().is_stm_gain_mode(Segment::S0));
            assert_eq!(segment, cpu.fpga().req_stm_segment());
            assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(Segment::S0));
            assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S0));
            assert_eq!(freq_div, cpu.fpga().stm_freq_division(Segment::S0));
            assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
            assert_eq!(
                (geometry[0].sound_speed / METER * 64.0).round() as u16,
                cpu.fpga().sound_speed(Segment::S0)
            );
            foci.iter().enumerate().for_each(|(focus_idx, focus)| {
                cpu.fpga()
                    .drives(Segment::S0, focus_idx)
                    .iter()
                    .enumerate()
                    .for_each(|(tr_idx, &drive)| {
                        let tr = cpu.fpga().local_tr_pos()[tr_idx];
                        let tx = ((tr >> 16) & 0xFFFF) as i32;
                        let ty = (tr & 0xFFFF) as i16 as i32;
                        let tz = 0;
                        let fx = (focus[0].point().x / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let fy = (focus[0].point().y / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let fz = (focus[0].point().z / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let d =
                            ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt() as u32;
                        let q = (d << 14) / cpu.fpga().sound_speed(Segment::S0) as u32;
                        let sin = (sin_table[q as usize % 256] >> 1) as usize;
                        let cos = (sin_table[(q as usize + 64) % 256] >> 1) as usize;
                        let p = atan_table[(sin << 7) | cos];
                        assert_eq!(Phase::new(p), drive.phase());
                        assert_eq!(focus.intensity(), drive.intensity());
                    })
            });
        }

        {
            let freq_div = rng.gen_range(
                SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT)..=0xFFFF,
            );
            let foci = gen_random_foci::<1>(2);
            let loop_behavior = LoopBehavior::once();
            let segment = Segment::S1;

            let stm = FociSTM::from_sampling_config(
                SamplingConfig::Division(NonZeroU16::new(freq_div).unwrap()),
                foci.clone(),
            )
            .with_loop_behavior(loop_behavior)
            .with_segment(segment, None);

            assert_eq!(Ok(()), send(&mut cpu, stm, &geometry, &mut tx));

            assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
            assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
            assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(Segment::S1));
            assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S1));
            assert_eq!(freq_div, cpu.fpga().stm_freq_division(Segment::S1));
            assert_eq!(TransitionMode::Immediate, cpu.fpga().stm_transition_mode());
            assert_eq!(
                (geometry[0].sound_speed / METER * 64.0).round() as u16,
                cpu.fpga().sound_speed(Segment::S1)
            );
            foci.iter().enumerate().for_each(|(focus_idx, focus)| {
                cpu.fpga()
                    .drives(Segment::S1, focus_idx)
                    .iter()
                    .enumerate()
                    .for_each(|(tr_idx, &drive)| {
                        let tr = cpu.fpga().local_tr_pos()[tr_idx];
                        let tx = ((tr >> 16) & 0xFFFF) as i32;
                        let ty = (tr & 0xFFFF) as i16 as i32;
                        let tz = 0;
                        let fx = (focus[0].point().x / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let fy = (focus[0].point().y / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let fz = (focus[0].point().z / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                        let d =
                            ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt() as u32;
                        let q = (d << 14) / cpu.fpga().sound_speed(Segment::S0) as u32;
                        let sin = (sin_table[q as usize % 256] >> 1) as usize;
                        let cos = (sin_table[(q as usize + 64) % 256] >> 1) as usize;
                        let p = atan_table[sin << 7 | cos];
                        assert_eq!(Phase::new(p), drive.phase());
                        assert_eq!(focus.intensity(), drive.intensity());
                    })
            });
        }

        {
            let d = SwapSegment::FociSTM(Segment::S1, TransitionMode::SyncIdx);

            assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

            assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());
            assert_eq!(TransitionMode::SyncIdx, cpu.fpga().stm_transition_mode());
        }
    });
}

#[test]
fn change_foci_stm_segment() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());

    let stm = FociSTM::from_sampling_config(
        SamplingConfig::Division(NonZeroU16::MAX),
        gen_random_foci::<1>(2),
    )
    .with_loop_behavior(LoopBehavior::infinite())
    .with_segment(Segment::S1, None);

    assert_eq!(Ok(()), send(&mut cpu, stm, &geometry, &mut tx));
    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());

    let d = SwapSegment::FociSTM(Segment::S1, TransitionMode::Immediate);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());
}

#[test]
fn test_foci_stm_freq_div_too_small() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let stm = FociSTM::from_sampling_config(SamplingConfig::FREQ_40K, gen_random_foci::<1>(2))
            .with_loop_behavior(LoopBehavior::infinite())
            .with_segment(Segment::S0, Some(TransitionMode::Immediate));

        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, stm, &geometry, &mut tx)
        );
    }

    {
        let g = TestGain {
            buf: geometry
                .iter()
                .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::null()).collect()))
                .collect(),
        };
        assert_eq!(Ok(()), send(&mut cpu, g, &geometry, &mut tx));

        let d = Silencer::from_completion_steps(
            SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        );
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let stm = FociSTM::from_sampling_config(
            SamplingConfig::Division(
                NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT))
                    .unwrap(),
            ),
            gen_random_foci::<1>(2),
        )
        .with_loop_behavior(LoopBehavior::infinite())
        .with_segment(Segment::S1, None);

        assert_eq!(Ok(()), send(&mut cpu, stm, &geometry, &mut tx));

        let d = Silencer::from_completion_steps(
            SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT * 2,
        );
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = SwapSegment::FociSTM(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }
}

#[test]
fn send_foci_stm_invalid_segment_transition() {
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

        assert_eq!(Ok(()), send(&mut cpu, g, &geometry, &mut tx));
    }

    // segment 1: GainSTM
    {
        let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..2)
            .map(|_| {
                geometry
                    .iter()
                    .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::null()).collect()))
                    .collect()
            })
            .collect();
        let stm = GainSTM::from_sampling_config(
            SamplingConfig::Division(NonZeroU16::MAX),
            bufs.iter().map(|buf| TestGain { buf: buf.clone() }),
        )
        .with_segment(Segment::S1, Some(TransitionMode::Immediate));

        assert_eq!(Ok(()), send(&mut cpu, stm, &geometry, &mut tx));
    }

    {
        let d = SwapSegment::FociSTM(Segment::S0, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );

        let d = SwapSegment::FociSTM(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }
}

#[test]
fn send_foci_stm_invalid_transition_mode() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0 to 0
    {
        let stm = FociSTM::from_sampling_config(
            SamplingConfig::Division(NonZeroU16::MAX),
            gen_random_foci::<1>(2),
        )
        .with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, stm, &geometry, &mut tx)
        );
    }

    // segment 0 to 1 immidiate
    {
        let stm = FociSTM::from_sampling_config(
            SamplingConfig::Division(NonZeroU16::MAX),
            gen_random_foci::<1>(2),
        )
        .with_loop_behavior(LoopBehavior::once())
        .with_segment(Segment::S1, Some(TransitionMode::Immediate));

        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, stm, &geometry, &mut tx)
        );
    }

    // Infinite but SyncIdx
    {
        let stm = FociSTM::from_sampling_config(
            SamplingConfig::Division(NonZeroU16::MAX),
            gen_random_foci::<1>(2),
        )
        .with_segment(Segment::S1, None);

        assert_eq!(Ok(()), send(&mut cpu, stm, &geometry, &mut tx));

        let d = SwapSegment::FociSTM(Segment::S1, TransitionMode::SyncIdx);
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }
}

#[rstest::rstest]
#[test]
#[case(Ok(()), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN-autd3_driver::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(1), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
fn test_miss_transition_time(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] systime: OffsetDateTime,
    #[case] transition_time: OffsetDateTime,
) {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let transition_mode = TransitionMode::SysTime(DcSysTime::from_utc(transition_time).unwrap());
    let stm = FociSTM::from_sampling_config(
        SamplingConfig::Division(NonZeroU16::MAX),
        gen_random_foci::<1>(2).into_iter(),
    )
    .with_loop_behavior(LoopBehavior::once())
    .with_segment(Segment::S1, Some(transition_mode));

    cpu.update_with_sys_time(DcSysTime::from_utc(systime).unwrap());
    assert_eq!(expect, send(&mut cpu, stm, &geometry, &mut tx));
    if expect.is_ok() {
        assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
    }
}

fn test_send_foci_stm_n<const N: usize>() {
    temp_env::with_var("AUTD3_ULTRASOUND_FREQ", Some("40000"), || {
        let sin_table = include_bytes!("sin.dat");
        let atan_table = include_bytes!("atan.dat");

        let mut rng = rand::thread_rng();

        let mut geometry = create_geometry(1);
        geometry.set_sound_speed(400e3);
        let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
        let mut tx = TxDatagram::new(geometry.num_devices());

        {
            let freq_div = rng.gen_range(
                SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT)..=0xFFFF,
            );
            let foci = gen_random_foci::<N>(1000);
            let loop_behavior = LoopBehavior::infinite();
            let segment = Segment::S0;
            let transition_mode = TransitionMode::Immediate;

            let stm = FociSTM::from_sampling_config(
                SamplingConfig::Division(NonZeroU16::new(freq_div).unwrap()),
                foci.clone(),
            )
            .with_loop_behavior(loop_behavior)
            .with_segment(segment, Some(transition_mode));

            assert_eq!(Ok(()), send(&mut cpu, stm, &geometry, &mut tx));

            assert!(!cpu.fpga().is_stm_gain_mode(Segment::S0));
            assert_eq!(segment, cpu.fpga().req_stm_segment());
            assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(Segment::S0));
            assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S0));
            assert_eq!(freq_div, cpu.fpga().stm_freq_division(Segment::S0));
            assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
            assert_eq!(
                (geometry[0].sound_speed / METER * 64.0).round() as u16,
                cpu.fpga().sound_speed(Segment::S0)
            );
            foci.iter().enumerate().for_each(|(focus_idx, focus)| {
                cpu.fpga()
                    .drives(Segment::S0, focus_idx)
                    .iter()
                    .enumerate()
                    .for_each(|(tr_idx, &drive)| {
                        let tr = cpu.fpga().local_tr_pos()[tr_idx];
                        let tx = ((tr >> 16) & 0xFFFF) as i32;
                        let ty = (tr & 0xFFFF) as i16 as i32;
                        let tz = 0;
                        let base_offset = focus[0].offset();
                        let (sin, cos) = focus.into_iter().fold((0, 0), |acc, f| {
                            let fx = (f.point().x / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                            let fy = (f.point().y / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                            let fz = (f.point().z / FOCI_STM_FIXED_NUM_UNIT).round() as i32;
                            let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt()
                                as u32;
                            let q = (d << 14) / cpu.fpga().sound_speed(Segment::S0) as u32
                                + (f.offset() - base_offset).value() as u32;
                            let sin = sin_table[q as usize % 256] as usize;
                            let cos = sin_table[(q as usize + 64) % 256] as usize;
                            (acc.0 + sin, acc.1 + cos)
                        });
                        let (sin, cos) = ((sin / N) >> 1, (cos / N) >> 1);
                        let p = atan_table[(sin << 7) | cos];
                        assert_eq!(Phase::new(p), drive.phase());
                        assert_eq!(focus.intensity(), drive.intensity());
                    })
            });
        }
    });
}

#[test]
fn test_send_foci_stm_2() {
    test_send_foci_stm_n::<2>()
}

#[test]
fn test_send_foci_stm_3() {
    test_send_foci_stm_n::<3>()
}

#[test]
fn test_send_foci_stm_4() {
    test_send_foci_stm_n::<4>()
}

#[test]
fn test_send_foci_stm_5() {
    test_send_foci_stm_n::<5>()
}

#[test]
fn test_send_foci_stm_6() {
    test_send_foci_stm_n::<6>()
}

#[test]
fn test_send_foci_stm_7() {
    test_send_foci_stm_n::<7>()
}

#[test]
fn test_send_foci_stm_8() {
    test_send_foci_stm_n::<8>()
}
