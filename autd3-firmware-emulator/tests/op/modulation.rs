use autd3_driver::{
    common::EmitIntensity,
    cpu::TxDatagram,
    fpga::{
        SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN, SILENCER_STEPS_INTENSITY_DEFAULT,
        SILENCER_STEPS_PHASE_DEFAULT,
    },
    operation::ModulationOp,
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

    let m: Vec<_> = (0..65536).map(|_| EmitIntensity::new(rng.gen())).collect();
    let freq_div = rng.gen_range(
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
            ..=SAMPLING_FREQ_DIV_MAX,
    );
    let mut op = ModulationOp::new(m.clone(), freq_div);

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(m.len(), cpu.fpga().modulation_cycle());
    assert_eq!(freq_div, cpu.fpga().modulation_frequency_division());
    assert_eq!(m, cpu.fpga().modulation());

    Ok(())
}
