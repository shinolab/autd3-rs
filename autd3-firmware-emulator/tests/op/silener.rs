use std::num::NonZeroU16;

use autd3_driver::{
    datagram::*,
    error::AUTDDriverError,
    firmware::{
        cpu::TxMessage,
        fpga::{SamplingConfig, SilencerTarget},
    },
    geometry::Point3,
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[test]
fn send_silencer_fixed_update_rate() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    unsafe {
        let config = FixedUpdateRate {
            intensity: NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
            phase: NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
        };
        let d = Silencer {
            config,
            target: SilencerTarget::Intensity,
        };

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
        let d = Silencer {
            config,
            target: SilencerTarget::PulseWidth,
        };

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(config, cpu.fpga().silencer_update_rate());
        assert!(cpu.fpga().silencer_fixed_update_rate_mode());
        assert_eq!(SilencerTarget::PulseWidth, cpu.fpga().silencer_target());
    }

    Ok(())
}

#[cfg(not(feature = "dynamic_freq"))]
#[test]
fn send_silencer_fixed_completion_time() {
    use autd3_driver::defined::ultrasound_period;

    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    {
        let config = FixedCompletionTime {
            intensity: ultrasound_period() * rng.gen_range(1..=10),
            phase: ultrasound_period() * rng.gen_range(1..=u8::MAX) as u32,
            strict_mode: true,
        };
        let d = Silencer {
            config,
            target: SilencerTarget::Intensity,
        };

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(
            (config.intensity.as_nanos() / ultrasound_period().as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().intensity.get()
        );
        assert_eq!(
            (config.phase.as_nanos() / ultrasound_period().as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().phase.get()
        );
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict_mode());
        assert_eq!(SilencerTarget::Intensity, cpu.fpga().silencer_target());
    }

    {
        let config = FixedCompletionTime {
            intensity: ultrasound_period() * rng.gen_range(1..=10),
            phase: ultrasound_period() * rng.gen_range(1..=u8::MAX) as u32,
            strict_mode: true,
        };
        let d = Silencer {
            config,
            target: SilencerTarget::PulseWidth,
        };
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(
            (config.intensity.as_nanos() / ultrasound_period().as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().intensity.get()
        );
        assert_eq!(
            (config.phase.as_nanos() / ultrasound_period().as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().phase.get()
        );
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict_mode());
        assert_eq!(SilencerTarget::PulseWidth, cpu.fpga().silencer_target());
    }
}

#[test]
fn send_silencer_fixed_completion_steps() {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    {
        let config = FixedCompletionSteps {
            intensity: NonZeroU16::new(rng.gen_range(1..=10)).unwrap(),
            phase: NonZeroU16::new(rng.gen_range(1..=u8::MAX) as u16).unwrap(),
            strict_mode: true,
        };
        let d = Silencer {
            config,
            target: SilencerTarget::Intensity,
        };

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(config, cpu.fpga().silencer_completion_steps());
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict_mode());
        assert_eq!(SilencerTarget::Intensity, cpu.fpga().silencer_target());
    }

    {
        let config = FixedCompletionSteps {
            intensity: NonZeroU16::new(rng.gen_range(1..=10)).unwrap(),
            phase: NonZeroU16::new(rng.gen_range(1..=u8::MAX) as u16).unwrap(),
            strict_mode: true,
        };
        let d = Silencer {
            config,
            target: SilencerTarget::PulseWidth,
        };
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
#[case(Err(AUTDDriverError::InvalidSilencerSettings), 2)]
#[cfg_attr(miri, ignore)]
fn silencer_completetion_steps_too_large_mod(
    #[case] expect: Result<(), AUTDDriverError>,
    #[case] steps_intensity: u16,
) -> anyhow::Result<()> {
    use crate::op::modulation::TestModulation;

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::MIN,
            phase: NonZeroU16::MIN,
            strict_mode: true,
        },
        target: SilencerTarget::Intensity,
    };
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    // Send modulation
    {
        let d = TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect(),
            sampling_config: SamplingConfig::FREQ_40K,
        };

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let steps_phase = 1;
    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::new(steps_intensity).unwrap(),
            phase: NonZeroU16::new(steps_phase).unwrap(),
            strict_mode: true,
        },
        target: SilencerTarget::Intensity,
    };

    assert_eq!(expect, send(&mut cpu, d, &geometry, &mut tx));

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), 1, 1)]
#[case(Err(AUTDDriverError::InvalidSilencerSettings), 2, 1)]
#[case(Err(AUTDDriverError::InvalidSilencerSettings), 1, 2)]
#[cfg_attr(miri, ignore)]
fn silencer_completetion_steps_too_large_stm(
    #[case] expect: Result<(), AUTDDriverError>,
    #[case] steps_intensity: u16,
    #[case] steps_phase: u16,
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::MIN,
            phase: NonZeroU16::MIN,
            strict_mode: true,
        },
        target: SilencerTarget::Intensity,
    };
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    // Send FociSTM
    {
        let d = FociSTM {
            foci: (0..2).map(|_| Point3::origin()).collect::<Vec<_>>(),
            config: SamplingConfig::FREQ_40K,
        };

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::new(steps_intensity).unwrap(),
            phase: NonZeroU16::new(steps_phase).unwrap(),
            strict_mode: true,
        },
        target: SilencerTarget::Intensity,
    };

    assert_eq!(expect, send(&mut cpu, d, &geometry, &mut tx));

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_silencer_fixed_completion_steps_permissive() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let config = FixedCompletionSteps {
        intensity: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
        phase: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
        strict_mode: false,
    };
    let d = Silencer {
        config,
        target: SilencerTarget::Intensity,
    };

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(
        config.intensity,
        cpu.fpga().silencer_completion_steps().intensity
    );
    assert_eq!(config.phase, cpu.fpga().silencer_completion_steps().phase);
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
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let config = FixedCompletionSteps {
        intensity: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
        phase: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
        strict_mode: false,
    };
    let d = Silencer {
        config,
        target: SilencerTarget::Intensity,
    };

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(
        config.intensity,
        cpu.fpga().silencer_completion_steps().intensity
    );
    assert_eq!(config.phase, cpu.fpga().silencer_completion_steps().phase);
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict_mode());
}
