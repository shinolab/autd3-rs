use std::hint::black_box;

use autd3::{gain::Null, prelude::BoxedDatagram};
use autd3_core::{derive::*, link::MsgId};
use autd3_driver::{
    autd3_device::AUTD3,
    datagram::{Datagram, Group},
    firmware::{cpu::TxMessage, operation::OperationHandler},
    geometry::{Geometry, Point3},
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

const TEST_SIZES: &[usize] = &[1, 10, 20];

fn without_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/without-group");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("Null", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g = black_box(Null {});
                    let generator = g
                        .operation_generator(geometry, &DeviceFilter::all_enabled())
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

fn group(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/group");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("GroupNull", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g = Group {
                        key_map: |dev| Some(dev.idx()),
                        datagram_map: geometry.iter().map(|dev| (dev.idx(), Null {})).collect(),
                    };
                    let generator = g
                        .operation_generator(geometry, &DeviceFilter::all_enabled())
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

fn group_boxed(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/group-boxed");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("GroupNullBoxed", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g = Group {
                        key_map: |dev| Some(dev.idx()),
                        datagram_map: geometry
                            .iter()
                            .map(|dev| (dev.idx(), BoxedDatagram::new(Null {})))
                            .collect(),
                    };
                    let generator = g
                        .operation_generator(geometry, &DeviceFilter::all_enabled())
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

fn gain_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/group/gain-group");

    TEST_SIZES.iter().for_each(|&size| {
        group.bench_with_input(
            BenchmarkId::new("GainGroupNull", size),
            &generate_geometry(size),
            |b, geometry| {
                let mut tx = vec![TxMessage::new_zeroed(); size];
                b.iter(|| {
                    let g = autd3::gain::Group {
                        key_map: |dev| {
                            let dev_idx = dev.idx();
                            move |_tr| Some(dev_idx)
                        },
                        gain_map: geometry.iter().map(|dev| (dev.idx(), Null {})).collect(),
                    };
                    let generator = g
                        .operation_generator(geometry, &DeviceFilter::all_enabled())
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

criterion_group!(benches, without_group, group, group_boxed, gain_group);
criterion_main!(benches);
