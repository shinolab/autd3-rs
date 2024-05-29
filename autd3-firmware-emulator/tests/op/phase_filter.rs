use autd3_driver::{datagram::PhaseFilter, derive::Phase, firmware::cpu::TxDatagram};
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
    let d = PhaseFilter::additive(|_| {
        let phase_offsets = phase_offsets.clone();
        move |tr| phase_offsets[tr.idx()]
    });

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(phase_offsets, cpu.fpga().phase_filter());

    Ok(())
}
