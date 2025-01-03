use std::collections::HashMap;

use autd3_derive::Gain;
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::{
        Datagram, DatagramS, Gain, GainContextGenerator, GainOperationGenerator, IntoBoxedGain,
    },
    defined::rad,
    error::AUTDDriverError,
    firmware::{
        cpu::TxMessage,
        fpga::{Drive, EmitIntensity, Phase, Segment, TransitionMode},
        operation::{GainContext, OperationHandler},
    },
    geometry::{Device, Geometry, IntoDevice, Point3, Transducer},
};

use bit_vec::BitVec;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zerocopy::FromZeros;

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3::new(Point3::new(i as f32 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i as _)
            })
            .collect(),
        4,
    )
}

#[derive(Gain, Clone, PartialEq, Debug)]
struct Focus {
    pos: Point3,
    intensity: EmitIntensity,
    phase_offset: Phase,
}

impl Focus {
    pub const fn new(pos: Point3) -> Self {
        Self {
            pos,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::ZERO,
        }
    }
}

struct FocusContext {
    pos: Point3,
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

    fn init(
        self,
        _geometry: &Geometry,
        _filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDDriverError> {
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
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g =
                        Focus::new(Point3::new(black_box(90.), black_box(70.), black_box(150.)));
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
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g =
                        Focus::new(Point3::new(black_box(90.), black_box(70.), black_box(150.)));
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
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g = Box::new(Focus::new(Point3::new(
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
