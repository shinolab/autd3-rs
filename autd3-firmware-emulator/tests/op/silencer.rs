use std::num::NonZeroU16;

use autd3_core::{
    firmware::SamplingConfig,
    link::{MsgId, TxMessage},
};
use autd3_driver::{datagram::*, error::AUTDDriverError, geometry::Point3};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn send_silencer_fixed_update_rate_unsafe() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    unsafe {
        let config = FixedUpdateRate {
            intensity: NonZeroU16::new_unchecked(rng.random_range(1..=u16::MAX)),
            phase: NonZeroU16::new_unchecked(rng.random_range(1..=u16::MAX)),
        };
        let d = Silencer { config };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        assert_eq!(config, cpu.fpga().silencer_update_rate());
        assert!(cpu.fpga().silencer_fixed_update_rate_mode());
    }

    unsafe {
        let config = FixedUpdateRate {
            intensity: NonZeroU16::new_unchecked(rng.random_range(1..=u16::MAX)),
            phase: NonZeroU16::new_unchecked(rng.random_range(1..=u16::MAX)),
        };
        let d = Silencer { config };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        assert_eq!(config, cpu.fpga().silencer_update_rate());
        assert!(cpu.fpga().silencer_fixed_update_rate_mode());
    }

    Ok(())
}

#[test]
fn send_silencer_fixed_completion_time_unsafe() {
    use autd3_driver::common::ULTRASOUND_PERIOD;

    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let config = FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * rng.random_range(1..=10),
            phase: ULTRASOUND_PERIOD * rng.random_range(1..=u8::MAX) as u32,
            strict: true,
        };
        let d = Silencer { config };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        assert_eq!(
            (config.intensity.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().intensity.get()
        );
        assert_eq!(
            (config.phase.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().phase.get()
        );
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict());
    }

    {
        let config = FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * rng.random_range(1..=10),
            phase: ULTRASOUND_PERIOD * rng.random_range(1..=u8::MAX) as u32,
            strict: true,
        };
        let d = Silencer { config };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        assert_eq!(
            (config.intensity.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().intensity.get()
        );
        assert_eq!(
            (config.phase.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as u16,
            cpu.fpga().silencer_completion_steps().phase.get()
        );
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict());
    }
}

#[test]
fn send_silencer_fixed_completion_steps_unsafe() {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let config = FixedCompletionSteps {
            intensity: NonZeroU16::new(rng.random_range(1..=10)).unwrap(),
            phase: NonZeroU16::new(rng.random_range(1..=u8::MAX) as u16).unwrap(),
            strict: true,
        };
        let d = Silencer { config };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        assert_eq!(config, cpu.fpga().silencer_completion_steps());
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict());
    }

    {
        let config = FixedCompletionSteps {
            intensity: NonZeroU16::new(rng.random_range(1..=10)).unwrap(),
            phase: NonZeroU16::new(rng.random_range(1..=u8::MAX) as u16).unwrap(),
            strict: true,
        };
        let d = Silencer { config };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        assert_eq!(config, cpu.fpga().silencer_completion_steps());
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict());
    }
}

#[rstest::rstest]
#[case(Ok(()), 1)]
#[case(Err(AUTDDriverError::InvalidSilencerSettings), 2)]
fn silencer_completion_steps_too_large_mod(
    #[case] expect: Result<(), AUTDDriverError>,
    #[case] steps_intensity: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::op::modulation::TestModulation;

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::MIN,
            phase: NonZeroU16::MIN,
            strict: true,
        },
    };
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    // Send modulation
    {
        let d = TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect(),
            sampling_config: SamplingConfig::FREQ_40K,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    let steps_phase = 1;
    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::new(steps_intensity).unwrap(),
            phase: NonZeroU16::new(steps_phase).unwrap(),
            strict: true,
        },
    };

    assert_eq!(
        expect,
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    Ok(())
}

#[rstest::rstest]
#[case(Ok(()), 1, 1)]
#[case(Err(AUTDDriverError::InvalidSilencerSettings), 2, 1)]
#[case(Err(AUTDDriverError::InvalidSilencerSettings), 1, 2)]
fn silencer_completion_steps_too_large_stm(
    #[case] expect: Result<(), AUTDDriverError>,
    #[case] steps_intensity: u16,
    #[case] steps_phase: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::MIN,
            phase: NonZeroU16::MIN,
            strict: true,
        },
    };
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    // Send FociSTM
    {
        let d = FociSTM {
            foci: (0..2).map(|_| Point3::origin()).collect::<Vec<_>>(),
            config: SamplingConfig::FREQ_40K,
        };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    let d = Silencer {
        config: FixedCompletionSteps {
            intensity: NonZeroU16::new(steps_intensity).unwrap(),
            phase: NonZeroU16::new(steps_phase).unwrap(),
            strict: true,
        },
    };

    assert_eq!(
        expect,
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    Ok(())
}

#[test]
fn send_silencer_fixed_completion_steps_permissive() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    let config = FixedCompletionSteps {
        intensity: NonZeroU16::new(rng.random_range(1..=u16::MAX)).unwrap(),
        phase: NonZeroU16::new(rng.random_range(1..=u16::MAX)).unwrap(),
        strict: false,
    };
    let d = Silencer { config };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert_eq!(
        config.intensity,
        cpu.fpga().silencer_completion_steps().intensity
    );
    assert_eq!(config.phase, cpu.fpga().silencer_completion_steps().phase);
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict());

    Ok(())
}

#[test]
fn send_silencer_fixed_completion_time_permissive() {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    let config = FixedCompletionSteps {
        intensity: NonZeroU16::new(rng.random_range(1..=u16::MAX)).unwrap(),
        phase: NonZeroU16::new(rng.random_range(1..=u16::MAX)).unwrap(),
        strict: false,
    };
    let d = Silencer { config };

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert_eq!(
        config.intensity,
        cpu.fpga().silencer_completion_steps().intensity
    );
    assert_eq!(config.phase, cpu.fpga().silencer_completion_steps().phase);
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict());
}
