use autd3_driver::{
    datagram::PulseWidthEncoder,
    firmware::{cpu::TxMessage, fpga::PulseWidth},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[test]
fn config_pwe_unsafe() -> anyhow::Result<()> {
    let mut rng = rand::rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    {
        let buf: Vec<_> = (0..256)
            .map(|_| PulseWidth::new(rng.random_range(0..512)).unwrap())
            .collect();

        let d = PulseWidthEncoder::new(|_| |i| buf[i.0 as usize]);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(buf, cpu.fpga().pulse_width_encoder_table());
    }

    {
        let default_table: Vec<_> = (0..256)
            .map(|i| {
                PulseWidth::new(
                    ((i as f64 / 255.).asin() / std::f64::consts::PI * 512.0).round() as u16,
                )
                .unwrap()
            })
            .collect();

        let d = PulseWidthEncoder::default();

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(default_table, cpu.fpga().pulse_width_encoder_table());
    }

    Ok(())
}
