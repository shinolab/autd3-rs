use std::{collections::HashMap, time::Duration};

use autd3_driver::{
    datagram::Datagram,
    defined::{METER, MILLIMETER},
    derive::{Drive, LoopBehavior, Phase, Segment},
    error::AUTDInternalError,
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::{
        cpu::TxDatagram,
        fpga::{
            TransitionMode, FOCUS_STM_BUF_SIZE_MAX, FOCUS_STM_FIXED_NUM_UNIT,
            SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN, SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        },
        operation::{ControlPoint, FocusSTMChangeSegmentOp, FocusSTMOp, GainSTMMode, GainSTMOp},
    },
    geometry::Vector3,
};
use autd3_firmware_emulator::{cpu::params::SYS_TIME_TRANSITION_MARGIN, CPUEmulator};

use num_integer::Roots;
use rand::*;
use time::OffsetDateTime;

use crate::{create_geometry, op::gain::TestGain, send};

pub fn gen_random_foci(num: usize) -> Vec<ControlPoint> {
    let mut rng = rand::thread_rng();
    (0..num)
        .map(|_| {
            ControlPoint::new(Vector3::new(
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
            ))
            .with_intensity(rng.gen::<u8>())
        })
        .collect()
}

#[test]
fn test_send_focus_stm() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let mut geometry = create_geometry(1);
    geometry.set_sound_speed(400e3);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let foci = gen_random_foci(FOCUS_STM_BUF_SIZE_MAX);
        let loop_behaviour = LoopBehavior::Infinite;
        let segment = Segment::S0;
        let transition_mode = TransitionMode::SyncIdx;
        let mut op = FocusSTMOp::new(
            foci.clone(),
            freq_div,
            loop_behaviour,
            segment,
            Some(transition_mode),
        );

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert!(!cpu.fpga().is_stm_gain_mode(Segment::S0));
        assert_eq!(segment, cpu.fpga().current_stm_segment());
        assert_eq!(loop_behaviour, cpu.fpga().stm_loop_behavior(Segment::S0));
        assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S0));
        assert_eq!(freq_div, cpu.fpga().stm_frequency_division(Segment::S0));
        assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
        assert_eq!(
            (geometry[0].sound_speed / METER * 1024.0).round() as u32,
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
                    let fx = (focus.point().x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                    let fy = (focus.point().y / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                    let fz = (focus.point().z / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                    let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt() as u64;
                    let q = (d << 18) / cpu.fpga().sound_speed(Segment::S0) as u64;
                    assert_eq!(Phase::new((q & 0xFF) as u8), drive.phase());
                    assert_eq!(focus.intensity(), drive.intensity());
                })
        });
    }

    {
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let foci = gen_random_foci(2);
        let loop_behaviour = LoopBehavior::once();
        let segment = Segment::S1;
        let mut op = FocusSTMOp::new(foci.clone(), freq_div, loop_behaviour, segment, None);

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
        assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());
        assert_eq!(loop_behaviour, cpu.fpga().stm_loop_behavior(Segment::S1));
        assert_eq!(foci.len(), cpu.fpga().stm_cycle(Segment::S1));
        assert_eq!(freq_div, cpu.fpga().stm_frequency_division(Segment::S1));
        assert_eq!(TransitionMode::SyncIdx, cpu.fpga().stm_transition_mode());
        assert_eq!(
            (geometry[0].sound_speed / METER * 1024.0).round() as u32,
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
                    let fx = (focus.point().x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                    let fy = (focus.point().y / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                    let fz = (focus.point().z / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                    let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt() as u64;
                    let q = (d << 18) / cpu.fpga().sound_speed(Segment::S1) as u64;
                    assert_eq!(Phase::new((q & 0xFF) as u8), drive.phase());
                    assert_eq!(focus.intensity(), drive.intensity());
                })
        });
    }

    Ok(())
}

#[test]
fn change_focus_stm_segment() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());
    let mut op = FocusSTMOp::new(
        gen_random_foci(2),
        SAMPLING_FREQ_DIV_MAX,
        LoopBehavior::Infinite,
        Segment::S1,
        None,
    );
    send(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());

    let mut op = FocusSTMChangeSegmentOp::new(Segment::S1, TransitionMode::default());
    send(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert!(!cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(Segment::S1, cpu.fpga().current_stm_segment());

    Ok(())
}

#[test]
fn test_focus_stm_freq_div_too_small() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let mut op = FocusSTMOp::new(
        gen_random_foci(2),
        SAMPLING_FREQ_DIV_MIN,
        LoopBehavior::Infinite,
        Segment::S0,
        Some(TransitionMode::SyncIdx),
    );

    assert_eq!(
        Err(AUTDInternalError::FrequencyDivisionTooSmall),
        send(&mut cpu, &mut op, &geometry, &mut tx)
    );
}

#[test]
fn send_focus_stm_invalid_segment_transition() -> anyhow::Result<()> {
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
        let mut op = GainSTMOp::new(
            bufs.iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            0xFFFFFFFF,
            LoopBehavior::Infinite,
            Segment::S1,
            Some(TransitionMode::SyncIdx),
        );

        send(&mut cpu, &mut op, &geometry, &mut tx)?;
    }

    {
        let mut op = FocusSTMChangeSegmentOp::new(Segment::S0, TransitionMode::default());
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );

        let mut op = FocusSTMChangeSegmentOp::new(Segment::S1, TransitionMode::default());
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN-1))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(1), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
fn test_miss_transition_time(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] systime: OffsetDateTime,
    #[case] transition_time: OffsetDateTime,
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let transition_mode = TransitionMode::SysTime(DcSysTime::from_utc(transition_time).unwrap());
    let mut op = FocusSTMOp::new(
        gen_random_foci(2),
        SAMPLING_FREQ_DIV_MAX,
        LoopBehavior::once(),
        Segment::S1,
        Some(transition_mode),
    );

    cpu.set_dc_sys_time(DcSysTime::from_utc(systime).unwrap());
    assert_eq!(expect, send(&mut cpu, &mut op, &geometry, &mut tx));
    if expect.is_ok() {
        assert_eq!(transition_mode, cpu.fpga().stm_transition_mode());
    }

    Ok(())
}
