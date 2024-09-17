use autd3_driver::{
    datagram::PhaseCorrection,
    firmware::{cpu::TxDatagram, fpga::Phase},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn phase_corr() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: Vec<_> = (0..geometry.num_transducers())
        .map(|_| Phase::new(rng.gen()))
        .collect();

    let d = PhaseCorrection::new(|_| |tr| buf[tr.idx()]);

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(buf, cpu.fpga().phase_correction());
    assert_eq!(
        buf,
        cpu.fpga()
            .drives()
            .into_iter()
            .map(|d| d.phase())
            .collect::<Vec<_>>()
    );

    Ok(())
}
