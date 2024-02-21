use autd3_driver::{cpu::TxDatagram, datagram::*};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn send_debug_output_idx() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let idx = rng.gen_range(0..geometry[0].num_transducers());
    let (mut op, _) = ConfigureDebugOutputIdx::new(|dev| Some(&dev[idx])).operation()?;

    assert!(cpu.fpga().debug_output_idx().is_none());

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(Some(idx as _), cpu.fpga().debug_output_idx());

    Ok(())
}
