use autd3::prelude::*;
use autd3_driver::geometry::IntoDevice;

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .flat_map(|i| {
                (0..size).map(move |j| {
                    AUTD3::new(Vector3::new(
                        i as float * AUTD3::DEVICE_WIDTH,
                        j as float * AUTD3::DEVICE_HEIGHT,
                        0.,
                    ))
                    .into_device(j + i * size)
                })
            })
            .collect(),
    )
}
