use autd3_driver::{cpu::TxDatagram, datagram::*};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_mod_delay() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let (mut op, _) = ConfigureModDelay::new(|_dev, tr| tr.idx() as u16).operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    cpu.fpga()
        .mod_delays()
        .iter()
        .enumerate()
        .for_each(|(i, &d)| {
            assert_eq!(i as u16, d);
        });

    Ok(())
}
