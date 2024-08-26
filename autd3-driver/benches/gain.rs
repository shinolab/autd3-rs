use autd3_driver::{
    autd3_device::AUTD3,
    datagram::{Group, IntoGainCache},
    defined::rad,
    derive::{Geometry, *},
    firmware::{
        cpu::TxDatagram,
        fpga::{EmitIntensity, Phase},
        operation::OperationHandler,
    },
    geometry::{IntoDevice, Transducer, Vector3},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3::new(Vector3::new(i as f32 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i)
            })
            .collect(),
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
    fn calc(&self, _geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
        let pos = self.pos;
        let intensity = self.intensity;
        let phase_offset = self.phase_offset;
        Ok(Self::transform(move |dev| {
            let wavenumber = dev.wavenumber();
            Box::new(move |tr: &Transducer| {
                (
                    Phase::from(-(pos - tr.position()).norm() * wavenumber * rad) + phase_offset,
                    intensity,
                )
            })
        }))
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

fn focus_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    [1, 10].iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Gain::FocusCache", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                let g = Focus::new(Vector3::new(
                    black_box(90.),
                    black_box(70.),
                    black_box(150.),
                ))
                .with_cache();
                b.iter(|| {
                    let generator = g.clone().operation_generator(geometry).unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, false).unwrap();
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
                    let g: Box<dyn Gain + Send + Sync> = Box::new(Focus::new(Vector3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    )));
                    let generator = g.operation_generator(geometry).unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, false).unwrap();
                })
            },
        );
    });
    group.finish();
}

fn focus_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    let size = 10;
    group.bench_with_input(
        "Gain::FocusGrouped (Many devices)",
        &generate_geometry(size),
        |b, geometry| {
            let mut tx = TxDatagram::new(size);
            b.iter(|| {
                let g = Group::new(|_| {
                    |tr| {
                        if tr.idx() < 249 / 2 {
                            Some(0)
                        } else {
                            Some(1)
                        }
                    }
                })
                .set(
                    0,
                    Focus::new(Vector3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    )),
                )
                .set(
                    1,
                    Focus::new(Vector3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    )),
                );
                let generator = g.operation_generator(geometry).unwrap();
                let mut operations = OperationHandler::generate(generator, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, false).unwrap();
            })
        },
    );

    let size = 10;
    group.bench_with_input(
        "Gain::FocusGrouped (Many Keys)",
        &generate_geometry(size),
        |b, geometry| {
            let mut tx = TxDatagram::new(size);
            b.iter(|| {
                let g = (0..geometry[0].num_transducers()).fold(
                    Group::new(|_| |tr| Some(tr.idx())),
                    |acc, i| {
                        acc.set(
                            i,
                            Focus::new(Vector3::new(
                                black_box(90.),
                                black_box(70.),
                                black_box(150.),
                            )),
                        )
                    },
                );
                let generator = g.operation_generator(geometry).unwrap();
                let mut operations = OperationHandler::generate(generator, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, false).unwrap();
            })
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    focus,
    focus_boxed,
    focus_parallel,
    focus_cache,
    focus_group
);
criterion_main!(benches);
