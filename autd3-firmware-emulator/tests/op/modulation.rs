use std::{num::NonZeroU16, time::Duration};

use autd3_core::{
    common::{MOD_BUF_SIZE_MIN, SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
    datagram::GPIOIn,
    derive::*,
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    datagram::{FixedCompletionSteps, Silencer, SwapSegment, WithLoopBehavior, WithSegment},
    error::AUTDDriverError,
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::v12_1::fpga::MOD_BUF_SIZE_MAX,
};
use autd3_firmware_emulator::{CPUEmulator, cpu::params::SYS_TIME_TRANSITION_MARGIN};

use time::OffsetDateTime;

use rand::*;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[derive(Modulation, Debug)]
pub struct TestModulation {
    pub buf: Vec<u8>,
    pub sampling_config: SamplingConfig,
}

impl Modulation for TestModulation {
    fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
        Ok(self.buf.clone())
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.sampling_config
    }
}

#[rstest::rstest]
#[test]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MAX,
    LoopBehavior::Infinite,
    Segment::S0,
    Some(TransitionMode::Immediate)
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::Infinite,
    Segment::S0,
    Some(TransitionMode::Ext)
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::ONCE,
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I0))
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::ONCE,
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I1))
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::ONCE,
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I2))
)]
#[cfg_attr(miri, ignore)]
#[case(
    MOD_BUF_SIZE_MIN,
    LoopBehavior::ONCE,
    Segment::S1,
    Some(TransitionMode::GPIO(GPIOIn::I3))
)]
#[cfg_attr(miri, ignore)]
#[case(
    32768 + 1,
    LoopBehavior::Infinite,
    Segment::S0,
    Some(TransitionMode::Immediate)
)]
#[case(MOD_BUF_SIZE_MIN, LoopBehavior::ONCE, Segment::S1, None)]
fn send_mod_unsafe(
    #[case] n: usize,
    #[case] loop_behavior: LoopBehavior,
    #[case] segment: Segment,
    #[case] transition_mode: Option<TransitionMode>,
) -> anyhow::Result<()> {
    use autd3_core::link::TxMessage;

    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let m: Vec<_> = (0..n).map(|_| rng.random()).collect();
    let freq_div = rng.random_range(
        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT)..=u16::MAX,
    );
    let d = WithLoopBehavior::new(
        TestModulation {
            buf: m.clone(),
            sampling_config: SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
        },
        loop_behavior,
        segment,
        transition_mode,
    );

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert_eq!(m.len(), cpu.fpga().modulation_cycle(segment));
    assert_eq!(freq_div, cpu.fpga().modulation_freq_divide(segment));
    assert_eq!(loop_behavior, cpu.fpga().modulation_loop_behavior(segment));
    if let Some(transition_mode) = transition_mode {
        assert_eq!(segment, cpu.fpga().req_modulation_segment());
        assert_eq!(transition_mode, cpu.fpga().modulation_transition_mode());
    } else {
        assert_eq!(Segment::S0, cpu.fpga().req_modulation_segment());
    }
    assert_eq!(m, cpu.fpga().modulation_buffer(segment));

    Ok(())
}

#[test]
fn swap_mod_segment_unsafe() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let m: Vec<_> = (0..MOD_BUF_SIZE_MIN).map(|_| 0x00).collect();
    let freq_div = SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT);
    let d = WithLoopBehavior {
        inner: TestModulation {
            buf: m.clone(),
            sampling_config: SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
        },
        segment: Segment::S1,
        transition_mode: None,
        loop_behavior: LoopBehavior::Infinite,
    };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert_eq!(Segment::S0, cpu.fpga().req_modulation_segment());

    let d = SwapSegment::Modulation(Segment::S1, TransitionMode::Immediate);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert_eq!(Segment::S1, cpu.fpga().req_modulation_segment());

    Ok(())
}

#[test]
fn mod_freq_div_too_small() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let d = TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect::<Vec<_>>(),
            sampling_config: SamplingConfig::FREQ_40K,
        };
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        )
    }

    {
        let d = TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect::<Vec<_>>(),
            sampling_config: SamplingConfig::new(NonZeroU16::MAX),
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
            inner: TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect::<Vec<_>>(),
                sampling_config: SamplingConfig::new(
                    NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT).unwrap(),
                ),
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
                intensity: NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT * 2).unwrap(),
                phase: NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT).unwrap(),
                strict: true,
            },
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegment::Modulation(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_mod_invalid_transition_mode() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    // segment 0 to 0
    {
        let d = WithSegment {
            inner: TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect::<Vec<_>>(),
                sampling_config: SamplingConfig::FREQ_4K,
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
            inner: TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect::<Vec<_>>(),
                sampling_config: SamplingConfig::FREQ_4K,
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
            inner: TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect::<Vec<_>>(),
                sampling_config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: None,
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegment::Modulation(Segment::S1, TransitionMode::SyncIdx);
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
#[case(Err(AUTDDriverError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN)-autd3_driver::ethercat::EC_CYCLE_TIME_BASE)]
#[case(Err(AUTDDriverError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(1), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
fn test_miss_transition_time(
    #[case] expect: Result<(), AUTDDriverError>,
    #[case] systime: OffsetDateTime,
    #[case] transition_time: OffsetDateTime,
) -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let transition_mode = TransitionMode::SysTime(DcSysTime::from_utc(transition_time).unwrap());

    let d = WithLoopBehavior {
        inner: TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect::<Vec<_>>(),
            sampling_config: SamplingConfig::FREQ_4K,
        },
        segment: Segment::S1,
        transition_mode: Some(transition_mode),
        loop_behavior: LoopBehavior::ONCE,
    };

    cpu.update_with_sys_time(DcSysTime::from_utc(systime).unwrap());
    assert_eq!(
        expect,
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    if expect.is_ok() {
        assert_eq!(transition_mode, cpu.fpga().modulation_transition_mode());
    }

    Ok(())
}
