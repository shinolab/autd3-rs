use autd3_driver::{
    autd3_device::AUTD3,
    defined::FREQ_40K,
    derive::{Geometry, *},
    firmware::{cpu::TxDatagram, operation::OperationHandler},
    geometry::{IntoDevice, Vector3},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

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

#[derive(Clone, PartialEq, Debug)]
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
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        let pos = self.pos;
        let intensity = self.intensity;
        let phase_offset = self.phase_offset;
        Ok(Self::transform(move |dev| {
            let wavenumber = dev.wavenumber();
            Box::new(move |tr: &Transducer| {
                Drive::new(
                    Phase::from((pos - tr.position()).norm() * wavenumber * rad) + phase_offset,
                    intensity,
                )
            })
        }))
    }
}

impl Datagram for Focus {
    type O1 = GainOp;
    type O2 = NullOp;
    type G = GainOperationGenerator;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        let g = self.calc(geometry)?;
        Ok(Self::G {
            g: Box::new(g),
            segment: Segment::S0,
            transition: true,
        })
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
                    let gen = g.operation_generator(geometry).unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, usize::MAX).unwrap();
                })
            },
        );
    });
    group.finish();
}

fn focus_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    [1, 10].iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Gain::FocusParallel", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let g = Focus::new(Vector3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    ));
                    let gen = g.operation_generator(geometry).unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                })
            },
        );
    });
    group.finish();
}

criterion_group!(benches, focus, focus_parallel);
criterion_main!(benches);
