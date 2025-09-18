use autd3_core::{
    firmware::GPIOIn,
    link::{MsgId, TxMessage},
};
use autd3_driver::datagram::*;
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[test]
fn send_gpio_in() -> Result<(), Box<dyn std::error::Error>> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    assert_eq!([false; 4], cpu.fpga().gpio_in());

    let d = EmulateGPIOIn::new(|_dev| |gpio| gpio == GPIOIn::I0 || gpio == GPIOIn::I3);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert_eq!([true, false, false, true], cpu.fpga().gpio_in());

    let d = EmulateGPIOIn::new(|_dev| |gpio| gpio == GPIOIn::I1 || gpio == GPIOIn::I2);
    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );
    assert_eq!([false, true, true, false], cpu.fpga().gpio_in());

    Ok(())
}
