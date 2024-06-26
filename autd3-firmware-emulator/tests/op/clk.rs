use autd3_driver::{
    datagram::ConfigureFPGAClock,
    defined::{Freq, Hz},
    derive::Datagram,
    error::AUTDInternalError,
    firmware::{cpu::TxDatagram, operation::OperationHandler},
};
use autd3_firmware_emulator::{cpu::params::CLK_FLAG_END, CPUEmulator};

use crate::{create_geometry, send};

#[rstest::rstest]
#[test]
#[case(vec![
    0x280000ffff,
    0x980003800,
    0x81000038e,
    0xa10000041,
    0xbfc000040,
    0xc10000041,
    0xdfc000040,
    0xe10000041,
    0xffc000040,
    0x1010000041,
    0x11fc000040,
    0x610000041,
    0x7c0002840,
    0x1210000000,
    0x13c0003040,
    0x16c0001041,
    0x14100002cb,
    0x1580004800,
    0x18fc0001a9,
    0x1980007c01,
    0x1a80007fe9,
    0x4e66ff1100,
    0x4f666f9000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000001,
], 40000*Hz)]
#[case(vec![
    0x280000ffff,
    0x980002800,
    0x8100003cf,
    0xa10000041,
    0xbfc000040,
    0xc10000041,
    0xdfc000040,
    0xe10000041,
    0xffc000040,
    0x1010000041,
    0x11fc000040,
    0x610000041,
    0x7c0002840,
    0x1210000000,
    0x13c0003040,
    0x16c0001041,
    0x141000030c,
    0x1580005800,
    0x18fc000190,
    0x1980007c01,
    0x1a80007fe9,
    0x4e66ff1100,
    0x4f666f9000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000001,
], 41000*Hz)]
fn config_clk_40k(#[case] expect_rom: Vec<u64>, #[case] ultrasound_clk: Freq<u32>) {
    temp_env::with_var(
        "AUTD3_ULTRASOUND_FREQ",
        Some(ultrasound_clk.hz().to_string()),
        || {
            let geometry = create_geometry(1);
            let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
            let mut tx = TxDatagram::new(geometry.num_devices());

            let d = ConfigureFPGAClock::new();

            assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

            assert_eq!(expect_rom, cpu.fpga().drp_rom());
        },
    );
}

#[test]
fn config_clk_incomplete_data() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let d = ConfigureFPGAClock::new();
    let gen = d.operation_generator(&geometry)?;
    let mut op = OperationHandler::generate(gen, &geometry);
    OperationHandler::pack(&mut op, &geometry, &mut tx, usize::MAX)?;
    tx[0].payload[1] |= CLK_FLAG_END;
    tx[0].payload[2] |= 12;

    cpu.send(&tx);
    assert_eq!(
        Err(AUTDInternalError::IncompleteDrpRomData),
        Result::<(), AUTDInternalError>::from(&cpu.rx())
    );

    Ok(())
}
