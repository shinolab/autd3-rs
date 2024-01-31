use std::collections::HashMap;

use autd3_driver::{
    common::EmitIntensity, cpu::TxDatagram, datagram::*, derive::*, geometry::Geometry,
    operation::GainOp,
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[derive(Gain)]
pub(crate) struct TestGain {
    pub(crate) buf: HashMap<usize, Vec<Drive>>,
}

impl Gain for TestGain {
    fn calc(
        &self,
        _: &Geometry,
        _: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(self.buf.clone())
    }
}

#[test]
fn send_gain() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let buf: HashMap<usize, Vec<Drive>> = geometry
        .iter()
        .map(|dev| {
            (
                dev.idx(),
                dev.iter()
                    .map(|_| Drive {
                        phase: Phase::new(rng.gen()),
                        intensity: EmitIntensity::new(rng.gen()),
                    })
                    .collect(),
            )
        })
        .collect();
    let g = TestGain { buf: buf.clone() };

    let (mut op, _) = g.operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert!(!cpu.fpga().is_stm_mode());
    buf[&0]
        .iter()
        .zip(cpu.fpga().gain_drives())
        .for_each(|(&a, b)| {
            assert_eq!(a, b);
        });

    Ok(())
}
