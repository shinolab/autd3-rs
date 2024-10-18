use std::num::NonZeroU16;

use autd3_derive::Modulation;
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::*,
    defined::ULTRASOUND_PERIOD,
    derive::*,
    firmware::{
        cpu::TxDatagram,
        fpga::{
            Drive, EmitIntensity, Phase, SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        },
    },
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{
    create_geometry,
    op::{gain::TestGain, stm::foci::gen_random_foci},
    send,
};

#[derive(Modulation, Debug)]
struct TestMod {
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Modulation for TestMod {
    fn calc(self) -> Result<Vec<u8>, AUTDInternalError> {
        Ok(vec![u8::MIN; 100])
    }
}

#[test]
fn send_clear() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let d = Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD,
            phase: ULTRASOUND_PERIOD,
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
            SamplingConfig::new(
                SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u16,
            )
            .unwrap(),
            gen_random_foci::<1>(2),
        )?
        .with_segment(Segment::S0, Some(TransitionMode::Ext));
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
        FixedCompletionTime {
            intensity: SILENCER_STEPS_INTENSITY_DEFAULT * ULTRASOUND_PERIOD,
            phase: SILENCER_STEPS_PHASE_DEFAULT * ULTRASOUND_PERIOD,
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

    assert_eq!([0, 0, 0, 0], cpu.fpga().debug_types());
    assert_eq!([0, 0, 0, 0], cpu.fpga().debug_values());

    Ok(())
}
