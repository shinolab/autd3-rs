use criterion::{black_box, BenchmarkId, Criterion};

use autd3::prelude::*;
use autd3_driver::datagram::Gain;

use crate::generate_geometry;

pub fn focus(c: &mut Criterion) {
    let mut group = c.benchmark_group("autd3/gain/focus");

    (1..).take(5).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                b.iter(|| {
                    Focus::new(Vector3::new(
                        black_box(90.),
                        black_box(70.),
                        black_box(150.),
                    ))
                    .calc(geometry, GainFilter::All)
                    .unwrap();
                })
            },
        );
    });
    group.finish();
}
