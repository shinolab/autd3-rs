use autd3_core::{
    firmware::Phase,
    link::{MsgId, TxMessage},
};
use autd3_driver::datagram::PhaseCorrection;
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[test]
fn phase_corr_unsafe() -> anyhow::Result<()> {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let buf: Vec<_> = (0..geometry.num_transducers())
        .map(|_| Phase(rng.random()))
        .collect();

    let d = PhaseCorrection::new(|_| |tr| buf[tr.idx()]);

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert_eq!(buf, cpu.fpga().phase_correction());
    assert_eq!(
        buf,
        cpu.fpga()
            .drives()
            .into_iter()
            .map(|d| d.phase)
            .collect::<Vec<_>>()
    );

    Ok(())
}
