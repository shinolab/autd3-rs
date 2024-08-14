use autd3_driver::{datagram::*, firmware::cpu::TxDatagram};
use autd3_firmware_emulator::CPUEmulator;

use crate::{create_geometry, send};

#[rstest::rstest]
#[test]
#[case(0b10100000, true, true)]
#[case(0b00100000, true, false)]
#[case(0b10000000, false, true)]
#[case(0b00000000, false, false)]
#[cfg_attr(miri, ignore)]
fn send_cpu_gpio_out(
    #[case] expect: u8,
    #[case] pa5: bool,
    #[case] pa7: bool,
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = CpuGPIO::new(|_| CpuGPIOPort { pa5, pa7 });

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(expect, cpu.port_a_podr());

    Ok(())
}
