use autd3_driver::{cpu::TxDatagram, datagram::*, fpga::DebugType};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_debug_output_idx() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let debug_types = [
        DebugType::BaseSignal,
        DebugType::Direct(true),
        DebugType::Sync,
        DebugType::ForceFan,
    ];
    let (mut op, _) = ConfigureDebugSettings::new(|_| debug_types.clone()).operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(
        debug_types.clone().map(|ty| ty.ty()),
        cpu.fpga().debug_types()
    );
    assert_eq!(debug_types.map(|ty| ty.value()), cpu.fpga().debug_values());

    Ok(())
}
