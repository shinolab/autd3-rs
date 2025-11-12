use autd3_core::{
    ethercat::DcSysTime,
    firmware::{SamplingConfig, Segment, transition_mode::Immediate},
    link::{MsgId, TxMessage},
};
use autd3_driver::{datagram::*, firmware::fpga::FPGAState, geometry::Point3};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, op::modulation::TestModulation, send};

fn fpga_state(cpu: &CPUEmulator) -> FPGAState {
    unsafe { std::mem::transmute(cpu.rx().data()) }
}

#[test]
fn send_reads_fpga_state() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    assert!(!cpu.reads_fpga_state());

    let d = ReadsFPGAState::new(|_| true);

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert!(cpu.reads_fpga_state());
    assert_eq!(0, cpu.rx().data());

    cpu.fpga_mut().assert_thermal_sensor();
    cpu.update_with_sys_time(DcSysTime::ZERO);
    let state = fpga_state(&cpu);
    assert!(state.is_thermal_assert());
    assert!(state.is_gain_mode());
    assert!(!state.is_stm_mode());
    assert_eq!(Some(Segment::S0), state.current_gain_segment());
    assert_eq!(None, state.current_stm_segment());
    assert_eq!(Segment::S0, state.current_mod_segment());

    cpu.fpga_mut().deassert_thermal_sensor();
    cpu.update_with_sys_time(DcSysTime::ZERO);
    let state = fpga_state(&cpu);
    assert!(!state.is_thermal_assert());
    assert!(state.is_gain_mode());
    assert!(!state.is_stm_mode());
    assert_eq!(Some(Segment::S0), state.current_gain_segment());
    assert_eq!(None, state.current_stm_segment());
    assert_eq!(Segment::S0, state.current_mod_segment());

    {
        let d = WithSegment {
            inner: TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                sampling_config: SamplingConfig::new(std::num::NonZeroU16::MAX),
            },
            segment: Segment::S1,
            transition_mode: Immediate,
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = WithSegment {
            inner: FociSTM {
                foci: (0..2)
                    .map(|_| ControlPoint::from(Point3::origin()))
                    .collect::<Vec<_>>(),
                config: SamplingConfig::new(std::num::NonZeroU16::MAX),
            },
            segment: Segment::S1,
            transition_mode: Immediate,
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }
    cpu.update_with_sys_time(DcSysTime::ZERO);
    let state = fpga_state(&cpu);
    assert!(!state.is_thermal_assert());
    assert!(!state.is_gain_mode());
    assert!(state.is_stm_mode());
    assert_eq!(None, state.current_gain_segment());
    assert_eq!(Some(Segment::S1), state.current_stm_segment());
    assert_eq!(Segment::S1, state.current_mod_segment());

    Ok(())
}
