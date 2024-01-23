#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod controller;
pub mod error;
pub mod gain;
pub mod link;
pub mod modulation;
pub mod prelude;

pub use autd3_driver as driver;

pub use controller::Controller;

#[cfg(test)]
mod tests {
    use autd3_driver::{defined::float, geometry::Vector3};

    pub fn random_vector3(
        range_x: std::ops::Range<float>,
        range_y: std::ops::Range<float>,
        range_z: std::ops::Range<float>,
    ) -> Vector3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Vector3::new(
            rng.gen_range(range_x),
            rng.gen_range(range_y),
            rng.gen_range(range_z),
        )
    }
}
