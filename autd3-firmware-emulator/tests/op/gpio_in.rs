use autd3_driver::{datagram::*, firmware::cpu::TxDatagram};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_gpio_in() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert_eq!([false; 4], cpu.fpga().gpio_in());

    let (mut op, _) = EmulateGPIOIn::new(|_dev| [true, false, false, true]).operation();
    send(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!([true, false, false, true], cpu.fpga().gpio_in());

    let (mut op, _) = EmulateGPIOIn::new(|_dev| [false, true, true, false]).operation();
    send(&mut cpu, &mut op, &geometry, &mut tx)?;
    assert_eq!([false, true, true, false], cpu.fpga().gpio_in());

    Ok(())
}
