use criterion::{black_box, BenchmarkId, Criterion};

use autd3::prelude::*;
use autd3_driver::datagram::Gain;

use crate::generate_geometry;

pub fn plane(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/plane");

    (1..).take(5).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                b.iter(|| {
                    Plane::new(Vector3::new(black_box(0.), black_box(0.), black_box(1.)))
                        .calc(geometry)
                        .unwrap();
                })
            },
        );
    });
    group.finish();
}
