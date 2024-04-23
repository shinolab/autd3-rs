use autd3_driver::firmware::{cpu::TxDatagram, operation::ConfigurePulseWidthEncoderOp};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[test]
fn config_pwe() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: Vec<_> = (0..65536).map(|_| rng.gen()).collect();
    let full_width_start = rng.gen();
    let mut op = ConfigurePulseWidthEncoderOp::new(buf.clone(), full_width_start);

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(
        full_width_start,
        cpu.fpga().pulse_width_encoder_full_width_start()
    );
    assert_eq!(buf, cpu.fpga().pulse_width_encoder_table());

    Ok(())
}
