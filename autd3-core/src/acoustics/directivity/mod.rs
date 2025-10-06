mod sphere;
mod t4010a1;

use crate::common::Angle;

pub use sphere::Sphere;
pub use t4010a1::T4010A1;

/// A trait representing the directivity of ultrasound transducer.
pub trait Directivity: Send + Sync {
    /// Calculates the directivity based on the given angle.
    ///
    /// # Arguments
    ///
    /// * `theta` - The angle between the axial direction and the target direction.
    #[must_use]
    fn directivity(theta: Angle) -> f32;
}
