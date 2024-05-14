pub mod controller;
pub mod error;
pub mod gain;
pub mod link;
pub mod modulation;
pub mod prelude;

pub use autd3_driver as driver;
pub use autd3_driver::derive;

pub use controller::Controller;

#[cfg(test)]
mod tests {
    use autd3_driver::{
        autd3_device::AUTD3,
        defined::FREQ_40K,
        geometry::{Geometry, IntoDevice, Vector3},
    };

    pub fn random_vector3(
        range_x: std::ops::Range<f64>,
        range_y: std::ops::Range<f64>,
        range_z: std::ops::Range<f64>,
    ) -> Vector3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Vector3::new(
            rng.gen_range(range_x),
            rng.gen_range(range_y),
            rng.gen_range(range_z),
        )
    }

    pub fn create_geometry(n: usize) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|i| AUTD3::new(Vector3::zeros()).into_device(i, FREQ_40K))
                .collect(),
            FREQ_40K,
        )
    }
}
