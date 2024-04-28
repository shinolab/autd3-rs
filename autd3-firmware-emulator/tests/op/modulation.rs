use std::time::Duration;

use autd3_driver::{
    derive::{LoopBehavior, Segment},
    error::AUTDInternalError,
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::{
        cpu::TxDatagram,
        fpga::{
            EmitIntensity, TransitionMode, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN,
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
        },
        operation::{ModulationChangeSegmentOp, ModulationOp},
    },
};
use autd3_firmware_emulator::{cpu::params::SYS_TIME_TRANSITION_MARGIN, CPUEmulator};

use time::OffsetDateTime;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn send_mod() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let m: Vec<_> = (0..32768).map(|_| EmitIntensity::new(rng.gen())).collect();
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let loop_behavior = LoopBehavior::infinite();
        let transition_mode = TransitionMode::SyncIdx;
        let mut op = ModulationOp::new(
            m.clone(),
            freq_div,
            loop_behavior,
            Segment::S0,
            Some(transition_mode),
        );

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert_eq!(Segment::S0, cpu.fpga().current_mod_segment());
        assert_eq!(m.len(), cpu.fpga().modulation_cycle(Segment::S0));
        assert_eq!(freq_div, cpu.fpga().modulation_freq_division(Segment::S0));
        assert_eq!(
            loop_behavior,
            cpu.fpga().modulation_loop_behavior(Segment::S0)
        );
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
        assert_eq!(m, cpu.fpga().modulation(Segment::S0));
    }

    {
        let m: Vec<_> = (0..2).map(|_| EmitIntensity::new(rng.gen())).collect();
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let loop_behavior = LoopBehavior::once();
        let mut op = ModulationOp::new(m.clone(), freq_div, loop_behavior, Segment::S1, None);

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert_eq!(Segment::S0, cpu.fpga().current_mod_segment());
        assert_eq!(m.len(), cpu.fpga().modulation_cycle(Segment::S1));
        assert_eq!(freq_div, cpu.fpga().modulation_freq_division(Segment::S1));
        assert_eq!(
            loop_behavior,
            cpu.fpga().modulation_loop_behavior(Segment::S1)
        );
        assert_eq!(TransitionMode::SyncIdx, cpu.fpga().mod_transition_mode());
        assert_eq!(m, cpu.fpga().modulation(Segment::S1));
    }

    {
        let mut op = ModulationChangeSegmentOp::new(Segment::S1, TransitionMode::default());

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert_eq!(Segment::S1, cpu.fpga().current_mod_segment());
    }

    {
        let transition_mode = TransitionMode::GPIO;
        let mut op = ModulationOp::new(
            (0..2).map(|_| EmitIntensity::MAX).collect(),
            SAMPLING_FREQ_DIV_MAX,
            LoopBehavior::infinite(),
            Segment::S0,
            Some(transition_mode),
        );
        send(&mut cpu, &mut op, &geometry, &mut tx)?;
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
    }

    {
        let transition_mode = TransitionMode::Ext;
        let mut op = ModulationOp::new(
            (0..2).map(|_| EmitIntensity::MAX).collect(),
            SAMPLING_FREQ_DIV_MAX,
            LoopBehavior::infinite(),
            Segment::S0,
            Some(transition_mode),
        );
        send(&mut cpu, &mut op, &geometry, &mut tx)?;
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
    }

    Ok(())
}

#[test]
fn mod_freq_div_too_small() {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let mut op = ModulationOp::new(
        (0..2).map(|_| EmitIntensity::MAX).collect(),
        SAMPLING_FREQ_DIV_MIN,
        LoopBehavior::infinite(),
        Segment::S0,
        Some(TransitionMode::SyncIdx),
    );

    assert_eq!(
        Err(AUTDInternalError::FrequencyDivisionTooSmall),
        send(&mut cpu, &mut op, &geometry, &mut tx)
    )
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
    let mut op = ModulationOp::new(
        (0..2).map(|_| EmitIntensity::MAX).collect(),
        SAMPLING_FREQ_DIV_MAX,
        LoopBehavior::once(),
        Segment::S1,
        Some(transition_mode),
    );

    cpu.set_dc_sys_time(DcSysTime::from_utc(systime).unwrap());
    assert_eq!(expect, send(&mut cpu, &mut op, &geometry, &mut tx));
    if expect.is_ok() {
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
    }

    Ok(())
}
