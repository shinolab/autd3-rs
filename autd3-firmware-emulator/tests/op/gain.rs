use std::collections::HashMap;

use autd3_driver::{
    autd3_device::AUTD3,
    common::EmitIntensity,
    cpu::TxDatagram,
    datagram::*,
    derive::*,
    geometry::{Geometry, IntoDevice, Vector3},
    operation::{GainOp, NullOp, OperationHandler},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

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
fn send_gain() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let buf: HashMap<usize, Vec<Drive>> = geometry
        .iter()
        .map(|dev| {
            (
                dev.idx(),
                dev.iter()
                    .map(|_| Drive {
                        phase: Phase::new(rng.gen_range(0..=u8::MAX)),
                        intensity: EmitIntensity::new(rng.gen_range(0..=u8::MAX)),
                    })
                    .collect(),
            )
        })
        .collect();
    let g = TestGain { buf: buf.clone() };

    let mut op = GainOp::new(g);
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();

    cpu.send(&tx);
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    assert!(!cpu.fpga().is_stm_mode());
    cpu.fpga()
        .intensities_and_phases(0)
        .iter()
        .zip(buf[&0].iter())
        .for_each(|(a, b)| {
            assert_eq!(a.0, b.intensity.value());
            assert_eq!(a.1, b.phase.value());
        });
}
