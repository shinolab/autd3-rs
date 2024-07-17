use std::num::NonZeroU16;

use autd3_driver::{
    datagram::*,
    defined::ControlPoint,
    derive::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
    firmware::{cpu::TxDatagram, fpga::FPGAState},
    geometry::Vector3,
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, op::modulation::TestModulation, send};

fn fpga_state(cpu: &CPUEmulator) -> FPGAState {
    unsafe { std::mem::transmute(cpu.rx().data()) }
}

#[test]
fn send_reads_fpga_state() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert!(!cpu.reads_fpga_state());

    let d = ReadsFPGAState::new(|_| true);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert!(cpu.reads_fpga_state());
    assert_eq!(0, cpu.rx().data());

    cpu.fpga_mut().assert_thermal_sensor();
    cpu.update();
    let state = fpga_state(&cpu);
    assert!(state.is_thermal_assert());
    assert!(state.is_gain_mode());
    assert!(!state.is_stm_mode());
    assert_eq!(Some(Segment::S0), state.current_gain_segment());
    assert_eq!(None, state.current_stm_segment());
    assert_eq!(Segment::S0, state.current_mod_segment());

    cpu.fpga_mut().deassert_thermal_sensor();
    cpu.update();
    let state = fpga_state(&cpu);
    assert!(!state.is_thermal_assert());
    assert!(state.is_gain_mode());
    assert!(!state.is_stm_mode());
    assert_eq!(Some(Segment::S0), state.current_gain_segment());
    assert_eq!(None, state.current_stm_segment());
    assert_eq!(Segment::S0, state.current_mod_segment());

    {
        let d = TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect(),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S1, Some(TransitionMode::Immediate));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = FociSTM::from_sampling_config(
            SamplingConfig::Division(NonZeroU16::MAX),
            (0..2).map(|_| ControlPoint::new(Vector3::zeros())),
        )
        .with_segment(Segment::S1, Some(TransitionMode::Immediate));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }
    cpu.update();
    let state = fpga_state(&cpu);
    assert!(!state.is_thermal_assert());
    assert!(!state.is_gain_mode());
    assert!(state.is_stm_mode());
    assert_eq!(None, state.current_gain_segment());
    assert_eq!(Some(Segment::S1), state.current_stm_segment());
    assert_eq!(Segment::S1, state.current_mod_segment());

    Ok(())
}
