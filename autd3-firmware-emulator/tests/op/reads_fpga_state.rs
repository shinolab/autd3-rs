use autd3_driver::{
    datagram::*,
    firmware::{
        cpu::TxMessage,
        fpga::{FPGAState, LoopBehavior, SamplingConfig, Segment, TransitionMode},
    },
    geometry::Point3,
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, op::modulation::TestModulation, send};

use zerocopy::FromZeros;

fn fpga_state(cpu: &CPUEmulator) -> FPGAState {
    unsafe { std::mem::transmute(cpu.rx().data()) }
}

#[test]
fn send_reads_fpga_state() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

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

        let d = FociSTM::new(
            SamplingConfig::FREQ_MIN,
            (0..2).map(|_| ControlPoint::from(Point3::origin())),
        )?
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
