use autd3_driver::{datagram::PulseWidthEncoder, firmware::cpu::TxDatagram};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn config_pwe() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    dbg!('a');
    {
        let buf: Vec<_> = (0..256).map(|_| rng.gen()).collect();

        let d = PulseWidthEncoder::new(|_| |i| buf[i as usize]);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        //     assert_eq!(
        //         buf.into_iter().map(|v| v as u8).collect::<Vec<_>>(),
        //         cpu.fpga().pulse_width_encoder_table()
        //     );
    }

    // dbg!('b');
    // {
    //     let default_table: Vec<_> = (0..256)
    //         .map(|i| ((i as f64 / 255.).asin() / std::f64::consts::PI * 256.0).round() as u8)
    //         .collect();

    //     let d = PulseWidthEncoder::default();

    //     assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    //     assert_eq!(
    //         default_table
    //             .into_iter()
    //             .map(|v| v as u8)
    //             .collect::<Vec<_>>(),
    //         cpu.fpga().pulse_width_encoder_table()
    //     );
    // }

    Ok(())
}
