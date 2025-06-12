use std::num::NonZeroU16;

use crate::{
    create_geometry,
    op::{gain::TestGain, modulation::TestModulation, stm::foci::gen_random_foci},
    send,
};
use autd3_core::{derive::*, link::MsgId};
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::*,
    firmware::{
        cpu::TxMessage,
        fpga::{
            Drive, EmitIntensity, Phase, PulseWidth, SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        },
    },
};
use autd3_firmware_emulator::CPUEmulator;

use zerocopy::FromZeros;

#[test]
fn send_clear_unsafe() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let d = Silencer {
            config: FixedCompletionSteps {
                intensity: NonZeroU16::MIN,
                phase: NonZeroU16::MIN,
                strict_mode: true,
            },
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = Silencer {
            config: FixedUpdateRate {
                intensity: NonZeroU16::new(1).unwrap(),
                phase: NonZeroU16::new(1).unwrap(),
            },
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect(),
            sampling_config: SamplingConfig::new(NonZeroU16::MAX),
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = TestGain {
            data: [(
                0,
                vec![
                    Drive {
                        phase: Phase(0xFF),
                        intensity: EmitIntensity::MAX
                    };
                    AUTD3::NUM_TRANS_IN_UNIT
                ],
            )]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = WithSegment {
            inner: FociSTM {
                foci: gen_random_foci::<1>(2),
                config: SamplingConfig::new(
                    NonZeroU16::new(
                        SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT),
                    )
                    .unwrap(),
                ),
            },
            segment: Segment::S0,
            transition_mode: Some(TransitionMode::Ext),
        };
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = PhaseCorrection::new(|_| |_| Phase::PI);
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = PulseWidthEncoder::new(|_| |_| PulseWidth::new(0xFF).unwrap());
        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    let d = Clear::new();

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert!(!cpu.reads_fpga_state());
    assert_eq!(
        FixedUpdateRate {
            intensity: NonZeroU16::new(256).unwrap(),
            phase: NonZeroU16::new(256).unwrap(),
        },
        cpu.fpga().silencer_update_rate()
    );
    assert_eq!(
        NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT).unwrap(),
        cpu.fpga().silencer_completion_steps().intensity
    );
    assert_eq!(
        NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT).unwrap(),
        cpu.fpga().silencer_completion_steps().phase
    );
    assert!(cpu.fpga().silencer_fixed_completion_steps_mode());

    assert_eq!(2, cpu.fpga().modulation_cycle(Segment::S0));
    assert_eq!(2, cpu.fpga().modulation_cycle(Segment::S1));
    assert_eq!(0xFFFF, cpu.fpga().modulation_freq_divide(Segment::S0));
    assert_eq!(0xFFFF, cpu.fpga().modulation_freq_divide(Segment::S1));
    assert_eq!(
        LoopBehavior::Infinite,
        cpu.fpga().modulation_loop_behavior(Segment::S0)
    );
    assert_eq!(
        LoopBehavior::Infinite,
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
    assert_eq!(0xFFFF, cpu.fpga().stm_freq_divide(Segment::S0));
    assert_eq!(0xFFFF, cpu.fpga().stm_freq_divide(Segment::S1));
    assert_eq!(
        LoopBehavior::Infinite,
        cpu.fpga().stm_loop_behavior(Segment::S0)
    );
    assert_eq!(
        LoopBehavior::Infinite,
        cpu.fpga().stm_loop_behavior(Segment::S1)
    );

    assert!(
        cpu.fpga()
            .phase_correction()
            .into_iter()
            .all(|v| v == Phase::ZERO)
    );

    assert_eq!(
        include_bytes!("asin.dat")
            .chunks(2)
            .map(|v| PulseWidth::new(u16::from_le_bytes([v[1], v[0]])).unwrap())
            .collect::<Vec<_>>(),
        cpu.fpga().pulse_width_encoder_table()
    );

    assert_eq!([0, 0, 0, 0], cpu.fpga().gpio_out_types());
    assert_eq!([0, 0, 0, 0], cpu.fpga().gpio_out_values());

    Ok(())
}
