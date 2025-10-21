use std::hint::black_box;

use autd3::gain::Null;
use autd3_core::{
    derive::*,
    devices::AUTD3,
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    datagram::Group,
    firmware::operation::{BoxedDatagram, OperationGenerator, OperationHandler},
    geometry::{Geometry, Point3},
};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

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

const TEST_SIZES: &[usize] = &[1, 10, 20];

fn without_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/without-group");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Null", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); size];
                b.iter(|| {
                    let g = black_box(Null {});
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

fn group(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/group");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("GroupNull", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); size];
                b.iter(|| {
                    let g = Group {
                        key_map: |dev| Some(dev.idx()),
                        datagram_map: geometry.iter().map(|dev| (dev.idx(), Null {})).collect(),
                    };
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

fn group_boxed(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/group-boxed");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("GroupNullBoxed", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); size];
                b.iter(|| {
                    let g = Group {
                        key_map: |dev| Some(dev.idx()),
                        datagram_map: geometry
                            .iter()
                            .map(|dev| (dev.idx(), BoxedDatagram::new(Null {})))
                            .collect(),
                    };
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

fn gain_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/gain-group");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("GainGroupNull", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new(); size];
                b.iter(|| {
                    let g = autd3::gain::Group::new(
                        |dev| move |_tr| Some(dev.idx()),
                        geometry.iter().map(|dev| (dev.idx(), Null {})).collect(),
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

criterion_group!(benches, without_group, group, group_boxed, gain_group);
criterion_main!(benches);
