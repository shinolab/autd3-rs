use autd3_core::link::MsgId;
use autd3_driver::{datagram::*, firmware::cpu::TxMessage};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[rstest::rstest]
#[test]
#[case(0b10100000, true, true)]
#[case(0b00100000, true, false)]
#[case(0b10000000, false, true)]
#[case(0b00000000, false, false)]
fn send_cpu_gpio_out(
    #[case] expect: u8,
    #[case] pa5: bool,
    #[case] pa7: bool,
) -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let d = CpuGPIOOutputs::new(|_| CpuGPIOPort::new(pa5, pa7));

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert_eq!(expect, cpu.port_a_podr());

    Ok(())
}
