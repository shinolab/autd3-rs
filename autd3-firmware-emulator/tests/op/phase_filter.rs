use autd3_driver::{
    derive::Phase,
    firmware::{cpu::TxDatagram, operation::ConfigurePhaseFilterOp},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn send_phase_filter() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let phase_offsets: Vec<_> = (0..cpu.num_transducers())
        .map(|_| Phase::new(rng.gen()))
        .collect();
    let mut op = ConfigurePhaseFilterOp::new(|_| |tr| phase_offsets[tr.idx()]);

    assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

    assert_eq!(phase_offsets, cpu.fpga().phase_filter());

    Ok(())
}
