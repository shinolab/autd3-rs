mod gain;

use autd3::prelude::*;
use autd3_driver::{defined::FREQ_40K, geometry::IntoDevice};

use gain::*;

use criterion::{criterion_group, criterion_main};

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3::new(Vector3::new(i as f32 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i)
            })
            .collect(),
        FREQ_40K,
    )
}

criterion_group!(benches, focus, bessel, plane);
criterion_main!(benches);
