use std::hint::black_box;

use autd3_core::{
    derive::*,
    devices::AUTD3,
    geometry::Vector3,
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    firmware::operation::{OperationGenerator, OperationHandler},
    geometry::{Geometry, Point3},
};

use autd3_gain_holo::*;

use criterion::{criterion_group, criterion_main, Criterion};

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

const NUM_FOCI: &[usize] = &[2, 10, 20];
const NUM_DEVICES: &[usize] = &[1, 10, 20];

fn greedy(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain-holo/greedy");

    itertools::iproduct!(NUM_DEVICES, NUM_FOCI).for_each(|(&num_dev, &num_foci)| {
        group.bench_with_input(
            format!("Gain::Greedy(NumDev={num_dev}, NumFoci={num_foci}"),
            &generate_geometry(num_dev),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); num_dev];
                let target_amp = 2.5e3 * geometry.num_devices() as f32 * Pa;
                b.iter(|| {
                    let foci = (0..num_foci)
                        .map(|i| {
                            let theta = 2. * std::f32::consts::PI * i as f32 / num_foci as f32;
                            let theta = black_box(theta);
                            (
                                geometry.center()
                                    + Vector3::new(
                                        black_box(theta.cos()),
                                        black_box(theta.sin()),
                                        black_box(150.),
                                    ) * 30.,
                                target_amp,
                            )
                        })
                        .collect::<Vec<_>>();
                    let g = Greedy::<autd3_gain_holo::T4010A1>::with_directivity(
                        foci,
                        Default::default(),
                    );
                    let mut generator = g
                        .operation_generator(
                            geometry,
                            &Environment::default(),
                            &DeviceMask::AllEnabled,
                        )
                        .unwrap();
                    let mut operations = geometry
                        .iter()
                        .map(|dev| generator.generate(dev))
                        .collect::<Vec<_>>();
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

fn gs(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain-holo/gs");

    itertools::iproduct!(NUM_DEVICES, NUM_FOCI).for_each(|(&num_dev, &num_foci)| {
        group.bench_with_input(
            format!("Gain::GS(NumDev={num_dev}, NumFoci={num_foci}"),
            &generate_geometry(num_dev),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); num_dev];
                let target_amp = 2.5e3 * geometry.num_devices() as f32 * Pa;
                b.iter(|| {
                    let foci = (0..num_foci)
                        .map(|i| {
                            let theta = 2. * std::f32::consts::PI * i as f32 / num_foci as f32;
                            let theta = black_box(theta);
                            (
                                geometry.center()
                                    + Vector3::new(
                                        black_box(theta.cos()),
                                        black_box(theta.sin()),
                                        black_box(150.),
                                    ) * 30.,
                                target_amp,
                            )
                        })
                        .collect::<Vec<_>>();
                    let g =
                        GS::<autd3_gain_holo::T4010A1>::with_directivity(foci, Default::default());
                    let mut generator = g
                        .operation_generator(
                            geometry,
                            &Environment::default(),
                            &DeviceMask::AllEnabled,
                        )
                        .unwrap();
                    let mut operations = geometry
                        .iter()
                        .map(|dev| generator.generate(dev))
                        .collect::<Vec<_>>();
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

fn gspat(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain-holo/gspat");

    itertools::iproduct!(NUM_DEVICES, NUM_FOCI).for_each(|(&num_dev, &num_foci)| {
        group.bench_with_input(
            format!("Gain::GSPAT(NumDev={num_dev}, NumFoci={num_foci}"),
            &generate_geometry(num_dev),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); num_dev];
                let target_amp = 2.5e3 * geometry.num_devices() as f32 * Pa;
                b.iter(|| {
                    let foci = (0..num_foci)
                        .map(|i| {
                            let theta = 2. * std::f32::consts::PI * i as f32 / num_foci as f32;
                            let theta = black_box(theta);
                            (
                                geometry.center()
                                    + Vector3::new(
                                        black_box(theta.cos()),
                                        black_box(theta.sin()),
                                        black_box(150.),
                                    ) * 30.,
                                target_amp,
                            )
                        })
                        .collect::<Vec<_>>();
                    let g = GSPAT::<autd3_gain_holo::T4010A1>::with_directivity(
                        foci,
                        Default::default(),
                    );
                    let mut generator = g
                        .operation_generator(
                            geometry,
                            &Environment::default(),
                            &DeviceMask::AllEnabled,
                        )
                        .unwrap();
                    let mut operations = geometry
                        .iter()
                        .map(|dev| generator.generate(dev))
                        .collect::<Vec<_>>();
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

fn naive(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain-holo/naive");

    itertools::iproduct!(NUM_DEVICES, NUM_FOCI).for_each(|(&num_dev, &num_foci)| {
        group.bench_with_input(
            format!("Gain::Naive(NumDev={num_dev}, NumFoci={num_foci}"),
            &generate_geometry(num_dev),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); num_dev];
                let target_amp = 2.5e3 * geometry.num_devices() as f32 * Pa;
                b.iter(|| {
                    let foci = (0..num_foci)
                        .map(|i| {
                            let theta = 2. * std::f32::consts::PI * i as f32 / num_foci as f32;
                            let theta = black_box(theta);
                            (
                                geometry.center()
                                    + Vector3::new(
                                        black_box(theta.cos()),
                                        black_box(theta.sin()),
                                        black_box(150.),
                                    ) * 30.,
                                target_amp,
                            )
                        })
                        .collect::<Vec<_>>();
                    let g = Naive::<autd3_gain_holo::T4010A1>::with_directivity(
                        foci,
                        Default::default(),
                    );
                    let mut generator = g
                        .operation_generator(
                            geometry,
                            &Environment::default(),
                            &DeviceMask::AllEnabled,
                        )
                        .unwrap();
                    let mut operations = geometry
                        .iter()
                        .map(|dev| generator.generate(dev))
                        .collect::<Vec<_>>();
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

criterion_group!(benches, greedy, gs, gspat, naive,);
criterion_main!(benches);
