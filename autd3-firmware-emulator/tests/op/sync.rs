use autd3_driver::{datagram::*, firmware::cpu::TxDatagram};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_sync() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let (mut op, _) = Synchronize::new().operation()?;
    assert!(!cpu.synchronized());

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert!(cpu.synchronized());

    Ok(())
}
