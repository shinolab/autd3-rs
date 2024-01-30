use autd3_derive::Modulation;
use autd3_driver::{
    cpu::TxDatagram,
    datagram::*,
    derive::*,
    fpga::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT, SILENCER_VALUE_MIN},
    operation::FocusSTMOp,
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, op::stm::focus::gen_random_foci, send};

#[derive(Modulation)]
struct TestMod {
    config: SamplingConfiguration,
}

impl Modulation for TestMod {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(vec![EmitIntensity::MIN; 100])
    }
}

#[derive(Gain)]
struct TestGain {}

impl Gain for TestGain {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<std::collections::HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |_, _| Drive {
            phase: Phase::new(0xFF),
            intensity: EmitIntensity::MAX,
        }))
    }
}

#[test]
fn send_clear() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let (mut op, _) =
            ConfigureSilencer::fixed_completion_steps(SILENCER_VALUE_MIN, SILENCER_VALUE_MIN)?
                .operation()?;
        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        let (mut op, _) =
            ConfigureSilencer::fixed_update_rate(SILENCER_VALUE_MIN, SILENCER_VALUE_MIN)?
                .operation()?;
        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        let (mut op, _) = TestMod {
            config: SamplingConfiguration::from_frequency_division(10000)?,
        }
        .operation()?;
        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        let (mut op, _) = TestGain {}.operation()?;
        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        let mut op = FocusSTMOp::new(
            gen_random_foci(2),
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
            None,
            None,
        );
        send(&mut cpu, &mut op, &geometry, &mut tx)?;
    }

    let (mut op, _) = Clear::new().operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

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

    assert_eq!(2, cpu.fpga().modulation_cycle());
    assert_eq!(5120, cpu.fpga().modulation_frequency_division());
    assert_eq!(vec![EmitIntensity::MAX; 2], cpu.fpga().modulation());

    assert!(!cpu.fpga().is_stm_gain_mode());
    assert_eq!(vec![Drive::null(); 249], cpu.fpga().gain_drives());

    assert_eq!(vec![0x0000; 249], cpu.fpga().mod_delays());

    assert!(cpu.fpga().debug_output_idx().is_none());

    Ok(())
}
