use std::num::NonZeroU32;

use autd3_driver::{
    common::EmitIntensity,
    cpu::TxDatagram,
    derive::{LoopBehavior, Segment},
    fpga::{
        SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN, SILENCER_STEPS_INTENSITY_DEFAULT,
        SILENCER_STEPS_PHASE_DEFAULT,
    },
    operation::{ModulationChangeSegmentOp, ModulationOp},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn send_mod() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let m: Vec<_> = (0..32768).map(|_| EmitIntensity::new(rng.gen())).collect();
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let loop_behavior = LoopBehavior::Infinite;
        let mut op = ModulationOp::new(m.clone(), freq_div, loop_behavior, Segment::S0, true);

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert_eq!(Segment::S0, cpu.fpga().current_mod_segment());
        assert_eq!(m.len(), cpu.fpga().modulation_cycle(Segment::S0));
        assert_eq!(
            freq_div,
            cpu.fpga().modulation_frequency_division(Segment::S0)
        );
        assert_eq!(loop_behavior, cpu.fpga().stm_loop_behavior(Segment::S0));
        assert_eq!(m, cpu.fpga().modulation(Segment::S0));
    }

    {
        let m: Vec<_> = (0..32768).map(|_| EmitIntensity::new(rng.gen())).collect();
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let loop_behavior = LoopBehavior::Finite(NonZeroU32::new(1).unwrap());
        let mut op = ModulationOp::new(m.clone(), freq_div, loop_behavior, Segment::S1, false);

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert_eq!(Segment::S0, cpu.fpga().current_mod_segment());
        assert_eq!(m.len(), cpu.fpga().modulation_cycle(Segment::S1));
        assert_eq!(
            freq_div,
            cpu.fpga().modulation_frequency_division(Segment::S1)
        );
        assert_eq!(
            loop_behavior,
            cpu.fpga().modulation_loop_behavior(Segment::S1)
        );
        assert_eq!(m, cpu.fpga().modulation(Segment::S1));
    }

    {
        let mut op = ModulationChangeSegmentOp::new(Segment::S1);

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert_eq!(Segment::S1, cpu.fpga().current_mod_segment());
    }

    Ok(())
}
