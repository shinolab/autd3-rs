mod gain;

use autd3::prelude::*;

use gain::*;

use criterion::{criterion_group, criterion_main};

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3::new(Vector3::new(i as f64 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i)
            })
            .collect(),
    )
}

criterion_group!(benches, focus, bessel, plane);
criterion_main!(benches);
