use autd3_driver::{
    autd3_device::AUTD3,
    datagram::IntoBoxedGain,
    defined::rad,
    derive::{Geometry, *},
    firmware::{
        cpu::TxDatagram,
        fpga::{EmitIntensity, Phase},
        operation::OperationHandler,
    },
    geometry::{IntoDevice, Vector3},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3::new(Vector3::new(i as f32 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i as _)
            })
            .collect(),
    )
}

#[derive(Gain, Clone, PartialEq, Debug)]
struct Focus {
    pos: Vector3,
    intensity: EmitIntensity,
    phase_offset: Phase,
}

impl Focus {
    pub const fn new(pos: Vector3) -> Self {
        Self {
            pos,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::ZERO,
        }
    }
}

struct FocusContext {
    pos: Vector3,
    intensity: EmitIntensity,
    phase_offset: Phase,
    wavenumber: f32,
}

impl GainContext for FocusContext {
    fn calc(&self, tr: &Transducer) -> Drive {
        Drive::new(
            Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad)
                + self.phase_offset,
            self.intensity,
        )
    }
}

impl GainContextGenerator for Focus {
    type Context = FocusContext;

    fn generate(&mut self, device: &Device) -> Self::Context {
        FocusContext {
            pos: self.pos,
            intensity: self.intensity,
            phase_offset: self.phase_offset,
            wavenumber: device.wavenumber(),
        }
    }
}

impl Gain for Focus {
    type G = Focus;

    fn init_with_filter(
        self,
        _geometry: &Geometry,
        _filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(self)
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
                    let generator = g.operation_generator(geometry).unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, false).unwrap();
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
                    let generator = g.operation_generator(geometry).unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, true).unwrap();
                })
            },
        );
    });
    group.finish();
}

fn focus_boxed(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    [1, 10].iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Gain::FocusBoxed", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let g = Box::new(Focus::new(Vector3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    )))
                    .into_boxed();
                    let generator = g.operation_generator(geometry).unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, false).unwrap();
                })
            },
        );
    });
    group.finish();
}

criterion_group!(benches, focus, focus_boxed, focus_parallel,);
criterion_main!(benches);
