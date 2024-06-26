use autd3_driver::{datagram::*, firmware::cpu::TxDatagram};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_force_fan() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert!(!cpu.fpga().is_force_fan());

    let d = ForceFan::new(|_dev| true);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    assert!(cpu.fpga().is_force_fan());

    let d = ForceFan::new(|_dev| false);
    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
    assert!(!cpu.fpga().is_force_fan());

    Ok(())
}
