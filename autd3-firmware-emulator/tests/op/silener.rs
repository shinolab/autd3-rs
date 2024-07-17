use std::time::Duration;

use autd3_driver::{
    datagram::*,
    derive::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
    error::AUTDInternalError,
    firmware::cpu::TxDatagram,
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

    let update_rate_intensity = rng.gen_range(1..=u16::MAX);
    let update_rate_phase = rng.gen_range(1..=u16::MAX);
    let d = Silencer::from_update_rate(update_rate_intensity, update_rate_phase);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(
        update_rate_intensity,
        cpu.fpga().silencer_update_rate_intensity()
    );
    assert_eq!(update_rate_phase, cpu.fpga().silencer_update_rate_phase());
    assert!(cpu.fpga().silencer_fixed_update_rate_mode());

    Ok(())
}

#[test]
fn send_silencer_fixed_completion_steps() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let steps_intensity = rng.gen_range(1..=10);
    let steps_phase = rng.gen_range(1..=u16::MAX);
    let d = Silencer::from_completion_steps(steps_intensity, steps_phase);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(
        steps_intensity,
        cpu.fpga().silencer_completion_steps_intensity()
    );
    assert_eq!(steps_phase, cpu.fpga().silencer_completion_steps_phase());
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(cpu.silencer_strict_mode());

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), 1)]
#[case(Err(AUTDInternalError::InvalidSilencerSettings), 2)]
fn silencer_completetion_steps_too_large_mod(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] steps_intensity: u16,
) -> anyhow::Result<()> {
    use crate::op::modulation::TestModulation;

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = Silencer::from_completion_steps(1, 1);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    // Send modulation
    {
        let d = TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect(),
            config: SamplingConfig::FREQ_40K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let steps_phase = 1;
    let d = Silencer::from_completion_steps(steps_intensity, steps_phase);

    assert_eq!(expect, send(&mut cpu, d, &geometry, &mut tx));

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), 1, 1)]
#[case(Err(AUTDInternalError::InvalidSilencerSettings), 2, 1)]
#[case(Err(AUTDInternalError::InvalidSilencerSettings), 1, 2)]
fn silencer_completetion_steps_too_large_stm(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] steps_intensity: u16,
    #[case] steps_phase: u16,
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = Silencer::from_completion_steps(1, 1);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    // Send FociSTM
    {
        let d = FociSTM::from_sampling_config(
            SamplingConfig::FREQ_40K,
            (0..2).map(|_| Vector3::zeros()),
        )
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let d = Silencer::from_completion_steps(steps_intensity, steps_phase);

    assert_eq!(expect, send(&mut cpu, d, &geometry, &mut tx));

    Ok(())
}

#[test]
fn send_silencer_fixed_completion_steps_permissive() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let steps_intensity = rng.gen_range(1..=u16::MAX);
    let steps_phase = rng.gen_range(1..=u16::MAX);
    let d = Silencer::from_completion_steps(steps_intensity, steps_phase).with_strict_mode(false);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(
        cpu.fpga().silencer_completion_steps_intensity(),
        steps_intensity
    );
    assert_eq!(steps_phase, cpu.fpga().silencer_completion_steps_phase());
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict_mode());

    Ok(())
}

#[test]
fn send_silencer_fixed_completion_time() {
    temp_env::with_var("AUTD3_ULTRASOUND_FREQ", Some("40000"), || {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);
        let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
        let mut tx = TxDatagram::new(geometry.num_devices());

        let time_intensity = rng.gen_range(1..=10) * Duration::from_micros(25);
        let time_phase = rng.gen_range(1..=u16::MAX) as u32 * Duration::from_micros(25);
        let d = Silencer::from_completion_time(time_intensity, time_phase);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(
            (time_intensity.as_micros() / 25) as u16,
            cpu.fpga().silencer_completion_steps_intensity()
        );
        assert_eq!(
            (time_phase.as_micros() / 25) as u16,
            cpu.fpga().silencer_completion_steps_phase()
        );
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(cpu.silencer_strict_mode());
    });
}

#[test]
fn send_silencer_fixed_completion_time_permissive() {
    temp_env::with_var("AUTD3_ULTRASOUND_FREQ", Some("40000"), || {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);
        let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
        let mut tx = TxDatagram::new(geometry.num_devices());

        let time_intensity = rng.gen_range(1..=u16::MAX) as u32 * Duration::from_micros(25);
        let time_phase = rng.gen_range(1..=u16::MAX) as u32 * Duration::from_micros(25);
        let d = Silencer::from_completion_time(time_intensity, time_phase).with_strict_mode(false);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(
            (time_intensity.as_micros() / 25) as u16,
            cpu.fpga().silencer_completion_steps_intensity(),
        );
        assert_eq!(
            (time_phase.as_micros() / 25) as u16,
            cpu.fpga().silencer_completion_steps_phase()
        );
        assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
        assert!(!cpu.silencer_strict_mode());
    });
}
