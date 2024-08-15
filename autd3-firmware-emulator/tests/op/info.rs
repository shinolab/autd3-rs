use autd3_driver::{
    datagram::*,
    firmware::{cpu::TxDatagram, version::FirmwareVersion},
};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_firminfo() -> anyhow::Result<()> {
    const EMULATOR_BIT: u8 = 1 << 7;

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // configure Reads FPGA Info
    {
        assert!(!cpu.reads_fpga_state());
        let d = ReadsFPGAState::new(|_| true);
        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));
        assert!(cpu.reads_fpga_state());
    }

    send(&mut cpu, FetchFirmInfo::CPUMajor, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MAJOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FetchFirmInfo::CPUMinor, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FetchFirmInfo::FPGAMajor, &geometry, &mut tx)?;
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FetchFirmInfo::FPGAMinor, &geometry, &mut tx)?;
    assert_eq!(FirmwareVersion::LATEST_VERSION_NUM_MINOR, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FetchFirmInfo::FPGAFunctions, &geometry, &mut tx)?;
    assert_eq!(EMULATOR_BIT, cpu.rx().data());
    assert!(!cpu.reads_fpga_state());

    send(&mut cpu, FetchFirmInfo::Clear, &geometry, &mut tx)?;
    assert!(cpu.reads_fpga_state());

    Ok(())
}
