use autd3_core::link::{MsgId, TxMessage};
use autd3_driver::datagram::*;
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[test]
fn send_force_fan() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    assert!(!cpu.fpga().is_force_fan());

    let d = ForceFan::new(|_dev| true);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert!(cpu.fpga().is_force_fan());

    let d = ForceFan::new(|_dev| false);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert!(!cpu.fpga().is_force_fan());

    Ok(())
}
