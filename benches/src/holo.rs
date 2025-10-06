use std::hint::black_box;

use autd3_core::{
    derive::*,
    geometry::Vector3,
    link::{MsgId, TxMessage},
};
use autd3_driver::{
    autd3_device::AUTD3,
    firmware::operation::{OperationGenerator, OperationHandler},
    geometry::{Geometry, Point3},
};

use autd3_gain_holo::{Greedy, Pa};

use criterion::{criterion_group, criterion_main, Criterion};
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

const NUM_FOCI: &[usize] = &[2, 10, 20];
const NUM_DEVICES: &[usize] = &[1, 10, 20];

fn greedy(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain-holo/greedy");

    NUM_DEVICES.iter().for_each(|&num_dev| {
        NUM_FOCI.iter().for_each(|&num_foci| {
            group.bench_with_input(
                format!("Gain::Greedy(NumDev={num_dev}, NumFoci={num_foci}"),
                &generate_geometry(num_dev),
                |b, geometry| {
                    let mut tx = vec![TxMessage::new_zeroed(); num_dev];
                    let target_amp = 2.5e3 * geometry.num_devices() as f32 * Pa;
                    b.iter(|| {
                        let foci = (0..num_foci)
                            .map(|i| {
                                let theta = 2. * std::f32::consts::PI * i as f32 / num_foci as f32;
                                let theta = black_box(theta);
                                (
                                    geometry.center()
                                        + 30.
                                            * Vector3::new(
                                                black_box(theta.cos()),
                                                black_box(theta.sin()),
                                                black_box(150.),
                                            ),
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
    });
    group.finish();
}

criterion_group!(benches, greedy,);
criterion_main!(benches);
