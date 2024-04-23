use autd3_driver::{datagram::*, firmware::cpu::TxDatagram};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_reads_fpga_state() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    assert!(!cpu.reads_fpga_state());

    let (mut op, _) = ConfigureReadsFPGAState::new(|_| true).operation().unwrap();

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert!(cpu.reads_fpga_state());
    assert_eq!(0, cpu.rx_data());

    cpu.fpga_mut().assert_thermal_sensor();
    cpu.update();
    assert_eq!(0x89, cpu.rx_data());

    cpu.fpga_mut().deassert_thermal_sensor();
    cpu.update();
    assert_eq!(0x88, cpu.rx_data());

    Ok(())
}
