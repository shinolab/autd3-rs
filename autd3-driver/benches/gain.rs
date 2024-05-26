use autd3_driver::{
    autd3_device::AUTD3,
    datagram::GainCalcFn,
    defined::FREQ_40K,
    derive::{GainFilter, Geometry, *},
    firmware::{cpu::TxDatagram, operation::OperationHandler},
    geometry::{IntoDevice, Vector3},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rayon::iter::{ParallelBridge, ParallelIterator};

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3::new(Vector3::new(i as f64 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i)
            })
            .collect(),
        FREQ_40K,
    )
}

#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Focus {
    pos: Vector3,
    intensity: EmitIntensity,
    phase_offset: Phase,
}

impl Focus {
    pub const fn new(pos: Vector3) -> Self {
        Self {
            pos,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::new(0),
        }
    }
}

impl Gain for Focus {
    fn calc<'a>(
        &'a self,
        _geometry: &'a Geometry,
        filter: GainFilter<'a>,
    ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
        Ok(Self::transform(
            filter,
            Box::new(move |dev| {
                let wavenumber = dev.wavenumber();
                Box::new(move |tr| {
                    Drive::new(
                        Phase::from((self.pos - tr.position()).norm() * wavenumber * rad)
                            + self.phase_offset,
                        self.intensity,
                    )
                })
            }),
        ))
    }
}

fn focus(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    [1, 10].iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Gain::Focus", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let g = Focus::new(Vector3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    ));
                    let gen = g.operation(geometry).unwrap();
                    geometry
                        .devices()
                        .zip(tx.iter_mut())
                        .par_bridge()
                        .for_each(|(dev, tx)| {
                            let (mut op1, mut op2) = gen(dev);
                            OperationHandler::pack(&mut op1, &mut op2, dev, tx).unwrap();
                        });
                })
            },
        );
    });
    group.finish();
}

criterion_group!(benches, focus);
criterion_main!(benches);
