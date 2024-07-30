use std::{num::NonZeroU16, time::Duration};

use autd3_driver::{
    datagram::{IntoDatagramWithSegmentTransition, Silencer, SwapSegment},
    derive::*,
    error::AUTDInternalError,
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::{
        cpu::TxDatagram,
        fpga::{
            GPIOIn, TransitionMode, MOD_BUF_SIZE_MAX, MOD_BUF_SIZE_MIN,
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
        },
    },
};
use autd3_firmware_emulator::{cpu::params::SYS_TIME_TRANSITION_MARGIN, CPUEmulator};

use time::OffsetDateTime;

use rand::*;

use crate::{create_geometry, send};

#[derive(Modulation)]
pub struct TestModulation {
    pub buf: Arc<Vec<u8>>,
    pub config: SamplingConfig,
    pub loop_behavior: LoopBehavior,
}

impl Modulation for TestModulation {
    fn calc(&self) -> ModulationCalcResult {
        Ok(self.buf.clone())
    }
}

#[rstest::rstest]
#[test]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MAX,
    LoopBehavior::infinite(),
    Segment::S0,
    Some(TransitionMode::Immediate)
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::infinite(),
    Segment::S0,
    Some(TransitionMode::Ext)
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::once(),
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I0))
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::once(),
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I1))
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::once(),
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I2))
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::once(),
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I3))
)]
#[case(MOD_BUF_SIZE_MIN, LoopBehavior::once(), Segment::S1, None)]
fn send_mod(
    #[case] n: usize,
    #[case] loop_behavior: LoopBehavior,
    #[case] segment: Segment,
    #[case] transition_mode: Option<TransitionMode>,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let m: Vec<_> = (0..n).map(|_| rng.gen()).collect();
    let freq_div = rng.gen_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u16..=u16::MAX,
    );
    let d = TestModulation {
        buf: Arc::new(m.clone()),
        config: SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
        loop_behavior,
    }
    .with_segment(segment, transition_mode);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(m.len(), cpu.fpga().modulation_cycle(segment));
    assert_eq!(freq_div, cpu.fpga().modulation_freq_division(segment));
    assert_eq!(loop_behavior, cpu.fpga().modulation_loop_behavior(segment));
    if let Some(transition_mode) = transition_mode {
        assert_eq!(segment, cpu.fpga().req_mod_segment());
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
    } else {
        assert_eq!(Segment::S0, cpu.fpga().req_mod_segment());
    }
    assert_eq!(m, cpu.fpga().modulation(segment));

    Ok(())
}

#[test]
fn swap_mod_segmemt() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let m: Vec<_> = (0..MOD_BUF_SIZE_MIN).map(|_| 0x00).collect();
    let freq_div = SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u16;
    let d = TestModulation {
        buf: Arc::new(m.clone()),
        config: SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
        loop_behavior: LoopBehavior::infinite(),
    }
    .with_segment(Segment::S1, None);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    assert_eq!(Segment::S0, cpu.fpga().req_mod_segment());

    let d = SwapSegment::Modulation(Segment::S1, TransitionMode::Immediate);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    assert_eq!(Segment::S1, cpu.fpga().req_mod_segment());

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn mod_freq_div_too_small() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let d = TestModulation {
            buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
            config: SamplingConfig::new(NonZeroU16::new(1).unwrap()),
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));

        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, d, &geometry, &mut tx)
        )
    }

    {
        let d = TestModulation {
            buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
            config: SamplingConfig::new(NonZeroU16::MAX),
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = Silencer::from_completion_time(
            Silencer::DEFAULT_COMPLETION_TIME_INTENSITY,
            Silencer::DEFAULT_COMPLETION_TIME_PHASE,
        );
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = TestModulation {
            buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
            config: SamplingConfig::new(
                NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT as _).unwrap(),
            ),
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S1, None);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = Silencer::from_completion_time(
            Silencer::DEFAULT_COMPLETION_TIME_PHASE * 2,
            Silencer::DEFAULT_COMPLETION_TIME_PHASE,
        );
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = SwapSegment::Modulation(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_mod_invalid_transition_mode() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0 to 0
    {
        let d = TestModulation {
            buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    // segment 0 to 1 immidiate
    {
        let d = TestModulation {
            buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::once(),
        }
        .with_segment(Segment::S1, Some(TransitionMode::Immediate));
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    // Infinite but SyncIdx
    {
        let d = TestModulation {
            buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S1, None);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = SwapSegment::Modulation(Segment::S1, TransitionMode::SyncIdx);
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN-autd3_driver::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(1), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
#[cfg_attr(miri, ignore)]
fn test_miss_transition_time(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] systime: OffsetDateTime,
    #[case] transition_time: OffsetDateTime,
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let transition_mode = TransitionMode::SysTime(DcSysTime::from_utc(transition_time).unwrap());
    let d = TestModulation {
        buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
        config: SamplingConfig::FREQ_4K,
        loop_behavior: LoopBehavior::once(),
    }
    .with_segment(Segment::S1, Some(transition_mode));

    cpu.update_with_sys_time(DcSysTime::from_utc(systime).unwrap());
    assert_eq!(expect, send(&mut cpu, d, &geometry, &mut tx));
    if expect.is_ok() {
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
    }

    Ok(())
}
