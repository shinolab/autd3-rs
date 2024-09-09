use std::num::NonZeroU16;

use autd3_driver::{
    datagram::*,
    defined::ULTRASOUND_PERIOD,
    derive::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
    error::AUTDInternalError,
    firmware::{cpu::TxDatagram, fpga::SilencerTarget},
    geometry::Vector3,
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn send_silencer_fixed_update_rate() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    unsafe {
        let config = FixedUpdateRate {
            intensity: NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
            phase: NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
        };
        let d = Silencer::new(config);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(config, cpu.fpga().silencer_update_rate());
        assert!(cpu.fpga().silencer_fixed_update_rate_mode());
        assert_eq!(SilencerTarget::Intensity, cpu.fpga().silencer_target());
    }

    unsafe {
        let config = FixedUpdateRate {
            intensity: NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
            phase: NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
        };
        let d = Silencer::new(config).with_target(SilencerTarget::PulseWidth);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(config, cpu.fpga().silencer_update_rate());
        assert!(cpu.fpga().silencer_fixed_update_rate_mode());
        assert_eq!(SilencerTarget::PulseWidth, cpu.fpga().silencer_target());
    }

    Ok(())
}

#[test]
fn send_silencer_fixed_completion_time() {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let config = FixedCompletionTime {
            intensity: rng.gen_range(1..=10) * ULTRASOUND_PERIOD,
            phase: rng.gen_range(1..=u8::MAX) as u32 * ULTRASOUND_PERIOD,
        };
        let d = Silencer::new(config);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(config, cpu.fpga().silencer_completion_steps());
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict_mode());
        assert_eq!(SilencerTarget::Intensity, cpu.fpga().silencer_target());
    }

    {
        let config = FixedCompletionTime {
            intensity: rng.gen_range(1..=10) * ULTRASOUND_PERIOD,
            phase: rng.gen_range(1..=u8::MAX) as u32 * ULTRASOUND_PERIOD,
        };
        let d = Silencer::new(config).with_target(SilencerTarget::PulseWidth);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(config, cpu.fpga().silencer_completion_steps());
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict_mode());
        assert_eq!(SilencerTarget::PulseWidth, cpu.fpga().silencer_target());
    }
}

#[rstest::rstest]
#[test]
#[case(Ok(()), 1)]
#[case(Err(AUTDInternalError::InvalidSilencerSettings), 2)]
#[cfg_attr(miri, ignore)]
fn silencer_completetion_steps_too_large_mod(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] steps_intensity: u32,
) -> anyhow::Result<()> {
    use std::sync::Arc;

    use crate::op::modulation::TestModulation;

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = Silencer::new(FixedCompletionTime {
        intensity: ULTRASOUND_PERIOD,
        phase: ULTRASOUND_PERIOD,
    });
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    // Send modulation
    {
        let d = TestModulation {
            buf: Arc::new((0..2).map(|_| u8::MAX).collect()),
            config: SamplingConfig::FREQ_40K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let steps_phase = 1;
    let d = Silencer::new(FixedCompletionTime {
        intensity: ULTRASOUND_PERIOD * steps_intensity,
        phase: ULTRASOUND_PERIOD * steps_phase,
    });

    assert_eq!(expect, send(&mut cpu, d, &geometry, &mut tx));

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), 1, 1)]
#[case(Err(AUTDInternalError::InvalidSilencerSettings), 2, 1)]
#[case(Err(AUTDInternalError::InvalidSilencerSettings), 1, 2)]
#[cfg_attr(miri, ignore)]
fn silencer_completetion_steps_too_large_stm(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] steps_intensity: u32,
    #[case] steps_phase: u32,
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = Silencer::new(FixedCompletionTime {
        intensity: ULTRASOUND_PERIOD,
        phase: ULTRASOUND_PERIOD,
    });
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    // Send FociSTM
    {
        let d = FociSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| Vector3::zeros()))?
            .with_segment(Segment::S0, Some(TransitionMode::Immediate));

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let d = Silencer::new(FixedCompletionTime {
        intensity: ULTRASOUND_PERIOD * steps_intensity,
        phase: ULTRASOUND_PERIOD * steps_phase,
    });

    assert_eq!(expect, send(&mut cpu, d, &geometry, &mut tx));

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_silencer_fixed_completion_steps_permissive() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let config = FixedCompletionTime {
        intensity: ULTRASOUND_PERIOD * rng.gen_range(1..=u8::MAX as u32),
        phase: ULTRASOUND_PERIOD * rng.gen_range(1..=u8::MAX as u32),
    };
    let d = Silencer::new(config).with_strict_mode(false);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(config, cpu.fpga().silencer_completion_steps());
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict_mode());

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_silencer_fixed_completion_time_permissive() {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let config = FixedCompletionTime {
        intensity: rng.gen_range(1..=u8::MAX) as u32 * ULTRASOUND_PERIOD,
        phase: rng.gen_range(1..=u8::MAX) as u32 * ULTRASOUND_PERIOD,
    };
    let d = Silencer::new(config).with_strict_mode(false);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(config, cpu.fpga().silencer_completion_steps(),);
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict_mode());
}
