use std::hint::black_box;

use autd3_core::{
    derive::*,
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    autd3_device::AUTD3,
    common::rad,
    datagram::BoxedGain,
    firmware::{
        driver::Driver,
        latest::{Latest, operation::OperationHandler},
    },
    geometry::{Device, Geometry, Point3, Transducer},
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use zerocopy::FromZeros;

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3 {
                    pos: Point3::new(i as f32 * AUTD3::DEVICE_WIDTH, 0., 0.),
                    ..Default::default()
                }
                .into()
            })
            .collect(),
    )
}

#[derive(Clone, PartialEq, Debug)]
struct FocusOption {
    intensity: Intensity,
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
                intensity: Intensity::MAX,
                phase_offset: Phase::ZERO,
            },
        }
    }
}

struct Impl {
    pos: Point3,
    intensity: Intensity,
    phase_offset: Phase,
    wavenumber: f32,
}

impl GainCalculator for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        Drive {
            phase: Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad)
                + self.phase_offset,
            intensity: self.intensity,
        }
    }
}

impl GainCalculatorGenerator for Focus {
    type Calculator = Impl;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        Impl {
            pos: self.pos,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: device.wavenumber(),
        }
    }
}

impl Gain for Focus {
    type G = Focus;

    fn init(self, _: &Geometry, _: &TransducerFilter) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

const TEST_SIZES: &[usize] = &[1, 10, 20];

fn focus(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Gain::Focus", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g =
                        Focus::new(Point3::new(black_box(90.), black_box(70.), black_box(150.)));
                    let generator = g
                        .operation_generator(
                            geometry,
                            &DeviceFilter::all_enabled(),
                            &Latest.firmware_limits(),
                        )
                        .unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(
                        MsgId::new(0x00),
                        &mut operations,
                        geometry,
                        &mut tx,
                        false,
                    )
                    .unwrap();
                })
            },
        );
    });
    group.finish();
}

fn focus_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Gain::FocusParallel", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g =
                        Focus::new(Point3::new(black_box(90.), black_box(70.), black_box(150.)));
                    let generator = g
                        .operation_generator(
                            geometry,
                            &DeviceFilter::all_enabled(),
                            &Latest.firmware_limits(),
                        )
                        .unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(
                        MsgId::new(0x00),
                        &mut operations,
                        geometry,
                        &mut tx,
                        true,
                    )
                    .unwrap();
                })
            },
        );
    });
    group.finish();
}

fn focus_boxed(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Gain::FocusBoxed", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g = BoxedGain::new(Focus::new(Point3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    )));
                    let generator = g
                        .operation_generator(
                            geometry,
                            &DeviceFilter::all_enabled(),
                            &Latest.firmware_limits(),
                        )
                        .unwrap();
                    let mut operations = OperationHandler::generate(generator, geometry);
                    OperationHandler::pack(
                        MsgId::new(0x00),
                        &mut operations,
                        geometry,
                        &mut tx,
                        false,
                    )
                    .unwrap();
                })
            },
        );
    });
    group.finish();
}

criterion_group!(benches, focus, focus_boxed, focus_parallel,);
criterion_main!(benches);
