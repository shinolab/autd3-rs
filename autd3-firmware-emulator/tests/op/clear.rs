use std::num::NonZeroU16;

use crate::{
    create_geometry,
    op::{gain::TestGain, stm::foci::gen_random_foci},
    send,
};
use autd3_core::derive::*;
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::*,
    firmware::{
        cpu::TxMessage,
        fpga::{
            Drive, EmitIntensity, Phase, SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        },
    },
};
use autd3_firmware_emulator::CPUEmulator;

use zerocopy::FromZeros;

#[derive(Modulation, Debug)]
struct TestMod {
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Modulation for TestMod {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        Ok(vec![u8::MIN; 100])
    }
}

#[test]
fn send_clear() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    {
        let d = Silencer::new(FixedCompletionSteps {
            intensity: NonZeroU16::MIN,
            phase: NonZeroU16::MIN,
        });
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = Silencer::new(FixedUpdateRate {
            intensity: NonZeroU16::new(1).unwrap(),
            phase: NonZeroU16::new(1).unwrap(),
        });
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = TestMod {
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = TestGain {
            data: [(
                0,
                vec![Drive::new(Phase::new(0xFF), EmitIntensity::MAX); AUTD3::NUM_TRANS_IN_UNIT],
            )]
            .into_iter()
            .collect(),
        }
        .with_segment(Segment::S0, Some(TransitionMode::Immediate));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = FociSTM::new(
            SamplingConfig::new(SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT))
                .unwrap(),
            gen_random_foci::<1>(2),
        )?
        .with_segment(Segment::S0, Some(TransitionMode::Ext));
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = PhaseCorrection::new(|_| |_| Phase::PI);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        let d = PulseWidthEncoder::new(|_| |_| 0xFF);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    }

    let d = Clear::new();

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert!(!cpu.reads_fpga_state());
    assert_eq!(
        FixedUpdateRate {
            intensity: NonZeroU16::new(256).unwrap(),
            phase: NonZeroU16::new(256).unwrap(),
        },
        cpu.fpga().silencer_update_rate()
    );
    assert_eq!(
        FixedCompletionSteps {
            intensity: NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT).unwrap(),
            phase: NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT).unwrap(),
        },
        cpu.fpga().silencer_completion_steps()
    );
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());

    assert_eq!(2, cpu.fpga().modulation_cycle(Segment::S0));
    assert_eq!(2, cpu.fpga().modulation_cycle(Segment::S1));
    assert_eq!(0xFFFF, cpu.fpga().modulation_freq_division(Segment::S0));
    assert_eq!(0xFFFF, cpu.fpga().modulation_freq_division(Segment::S1));
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().modulation_loop_behavior(Segment::S0)
    );
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().modulation_loop_behavior(Segment::S1)
    );
    assert_eq!(vec![u8::MAX; 2], cpu.fpga().modulation_buffer(Segment::S0));
    assert_eq!(vec![u8::MAX; 2], cpu.fpga().modulation_buffer(Segment::S1));

    assert!(cpu.fpga().is_stm_gain_mode(Segment::S0));
    assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
    assert_eq!(vec![Drive::NULL; 249], cpu.fpga().drives_at(Segment::S0, 0));
    assert_eq!(vec![Drive::NULL; 249], cpu.fpga().drives_at(Segment::S1, 0));
    assert_eq!(1, cpu.fpga().stm_cycle(Segment::S0));
    assert_eq!(1, cpu.fpga().stm_cycle(Segment::S1));
    assert_eq!(0xFFFF, cpu.fpga().stm_freq_division(Segment::S0));
    assert_eq!(0xFFFF, cpu.fpga().stm_freq_division(Segment::S1));
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().stm_loop_behavior(Segment::S0)
    );
    assert_eq!(
        LoopBehavior::infinite(),
        cpu.fpga().stm_loop_behavior(Segment::S1)
    );

    assert!(cpu
        .fpga()
        .phase_correction()
        .into_iter()
        .all(|v| v == Phase::ZERO));

    assert_eq!(
        include_bytes!("asin.dat").to_vec(),
        cpu.fpga().pulse_width_encoder_table()
    );

    assert_eq!([0, 0, 0, 0], cpu.fpga().debug_types());
    assert_eq!([0, 0, 0, 0], cpu.fpga().debug_values());

    Ok(())
}
