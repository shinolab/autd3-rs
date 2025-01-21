use std::collections::HashMap;

use autd3_core::derive::*;
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::{Datagram, IntoBoxedGain},
    defined::rad,
    firmware::{
        cpu::TxMessage,
        fpga::{Drive, EmitIntensity, Phase},
        operation::OperationHandler,
    },
    geometry::{Device, Geometry, IntoDevice, Point3, Transducer},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zerocopy::FromZeros;

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3 {
                    pos: Point3::new(i as f32 * AUTD3::DEVICE_WIDTH, 0., 0.),
                    ..Default::default()
                }
                .into_device(i as _)
            })
            .collect(),
    )
}

#[derive(Clone, PartialEq, Debug)]
struct FocusOption {
    intensity: EmitIntensity,
    phase_offset: Phase,
}

#[derive(Gain, Clone, PartialEq, Debug)]
struct Focus {
    pos: Point3,
    option: FocusOption,
}

impl Focus {
    pub const fn new(pos: Point3) -> Self {
        Self {
            pos,
            option: FocusOption {
                intensity: EmitIntensity::MAX,
                phase_offset: Phase::ZERO,
            },
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
        Drive {
            phase: Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad)
                + self.phase_offset,
            intensity: self.intensity,
        }
    }
}

impl GainContextGenerator for Focus {
    type Context = FocusContext;

    fn generate(&mut self, device: &Device) -> Self::Context {
        FocusContext {
            pos: self.pos,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: device.wavenumber(),
        }
    }
}

impl Gain for Focus {
    type G = Focus;

    fn init(self) -> Result<Self::G, GainError> {
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
                    let generator = g
                        .operation_generator(geometry, &DatagramOption::default())
                        .unwrap();
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
                    let generator = g
                        .operation_generator(geometry, &DatagramOption::default())
                        .unwrap();
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
                    let generator = g
                        .operation_generator(geometry, &DatagramOption::default())
                        .unwrap();
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
