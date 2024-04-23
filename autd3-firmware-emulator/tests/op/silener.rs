use autd3_driver::{datagram::*, firmware::cpu::TxDatagram};
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
    let (mut op, _) =
        ConfigureSilencer::fixed_update_rate(update_rate_intensity, update_rate_phase)?
            .operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(
        update_rate_intensity,
        cpu.fpga().silencer_update_rate_intensity()
    );
    assert_eq!(update_rate_phase, cpu.fpga().silencer_update_rate_phase());
    assert!(!cpu.fpga().silencer_fixed_completion_steps_mode());

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
    let (mut op, _) =
        ConfigureSilencer::fixed_completion_steps(steps_intensity, steps_phase)?.operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(
        steps_intensity,
        cpu.fpga().silencer_completion_steps_intensity()
    );
    assert_eq!(steps_phase, cpu.fpga().silencer_completion_steps_phase());
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(cpu.silencer_strict_mode());

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
    let (mut op, _) = ConfigureSilencer::fixed_completion_steps(steps_intensity, steps_phase)?
        .with_strict_mode(false)
        .operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(
        cpu.fpga().silencer_completion_steps_intensity(),
        steps_intensity
    );
    assert_eq!(steps_phase, cpu.fpga().silencer_completion_steps_phase());
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());
    assert!(!cpu.silencer_strict_mode());

    Ok(())
}
