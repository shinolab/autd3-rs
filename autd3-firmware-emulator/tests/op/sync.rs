use autd3_driver::{datagram::*, firmware::cpu::TxDatagram};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_sync() -> anyhow::Result<()> {
    #[cfg(feature = "dynamic_freq")]
    autd3_driver::set_ultrasound_freq(autd3_driver::defined::FREQ_40K);

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = Synchronize::new();
    assert!(!cpu.synchronized());

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert!(cpu.synchronized());

    Ok(())
}
