use autd3_core::link::MsgId;
use autd3_driver::{datagram::*, firmware::cpu::TxMessage};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[test]
fn send_sync() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let d = Synchronize::new();
    assert!(!cpu.synchronized());

    assert_eq!(
        Ok(()),
        send(&mut MsgId::new(0), &mut cpu, d, &geometry, &mut tx)
    );

    assert!(cpu.synchronized());

    Ok(())
}
