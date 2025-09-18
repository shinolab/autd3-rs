use autd3_core::link::{MsgId, TxMessage};
use autd3_driver::datagram::Synchronize;
use autd3_firmware_emulator::CPUEmulator;
use zerocopy::FromZeros;

use crate::{create_geometry, send};

#[test]
fn send_sync() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let d = Synchronize::new();
    assert!(!cpu.synchronized());

    assert_eq!(
        Ok(()),
        send(&mut MsgId::new(0), &mut cpu, d, &mut geometry, &mut tx)
    );

    assert!(cpu.synchronized());

    Ok(())
}
