use autd3_derive::Modulation;
use autd3_driver::{
    datagram::*,
    derive::*,
    firmware::{
        cpu::TxDatagram,
        fpga::{
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT, SILENCER_VALUE_MIN,
        },
    },
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, op::stm::foci::gen_random_foci, send};

#[derive(Modulation)]
struct TestMod {
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Modulation for TestMod {
    fn calc(&self) -> ModulationCalcResult {
        Ok(vec![u8::MIN; 100])
    }
}

#[derive(Gain)]
struct TestGain {}

impl Gain for TestGain {
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        Ok(Self::transform(|_| {
            |_| Drive::new(Phase::new(0xFF), EmitIntensity::MAX)
        }))
    }
}

#[test]
fn send_clear() -> anyhow::Result<()> {

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let d = Silencer::from_completion_steps(SILENCER_VALUE_MIN, SILENCER_VALUE_MIN);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = Silencer::from_update_rate(SILENCER_VALUE_MIN, SILENCER_VALUE_MIN);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = TestMod {
            config: SamplingConfig::DivisionRaw(5120),
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = TestGain {}.with_segment(Segment::S0, true);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = FociSTM::from_sampling_config(
            SamplingConfig::DivisionRaw(
                SAMPLING_FREQ_DIV_MIN
                    * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
            ),
            gen_random_foci::<1>(2),
        )
        .with_segment(Segment::S0, Some(TransitionMode::Ext));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let d = Clear::new();

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert!(!cpu.reads_fpga_state());
    assert_eq!(256, cpu.fpga().silencer_update_rate_intensity());
    assert_eq!(256, cpu.fpga().silencer_update_rate_phase());
    assert_eq!(
        SILENCER_STEPS_INTENSITY_DEFAULT,
        cpu.fpga().silencer_completion_steps_intensity()
    );
    assert_eq!(
        SILENCER_STEPS_PHASE_DEFAULT,
        cpu.fpga().silencer_completion_steps_phase()
    );
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());

    assert_eq!(2, cpu.fpga().modulation_cycle(Segment::S0));
    assert_eq!(2, cpu.fpga().modulation_cycle(Segment::S1));
    assert_eq!(5120, cpu.fpga().modulation_freq_division(Segment::S0));
    assert_eq!(5120, cpu.fpga().modulation_freq_division(Segment::S1));
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().modulation_loop_behavior(Segment::S0)
    );
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().modulation_loop_behavior(Segment::S1)
    );
    assert_eq!(vec![u8::MAX; 2], cpu.fpga().modulation(Segment::S0));
    assert_eq!(vec![u8::MAX; 2], cpu.fpga().modulation(Segment::S1));

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S0));
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(vec![Drive::null(); 249], cpu.fpga().drives(Segment::S0, 0));
    assert_eq!(vec![Drive::null(); 249], cpu.fpga().drives(Segment::S1, 0));
    assert_eq!(1, cpu.fpga().stm_cycle(Segment::S0));
    assert_eq!(1, cpu.fpga().stm_cycle(Segment::S1));
    assert_eq!(0xFFFFFFFF, cpu.fpga().stm_freq_division(Segment::S0));
    assert_eq!(0xFFFFFFFF, cpu.fpga().stm_freq_division(Segment::S1));
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().stm_loop_behavior(Segment::S0)
    );
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().stm_loop_behavior(Segment::S1)
    );

    assert_eq!([0, 0, 0, 0], cpu.fpga().debug_types());
    assert_eq!([0, 0, 0, 0], cpu.fpga().debug_values());

    Ok(())
}
