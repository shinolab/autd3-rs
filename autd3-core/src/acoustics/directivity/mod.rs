mod sphere;
mod t4010a1;

use crate::{
    common::{Angle, rad},
    geometry::{UnitVector3, Vector3},
};

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

    /// Calculates the directivity based on the axial direction and target direction.
    #[must_use]
    fn directivity_from_dir(axial_direction: &UnitVector3, target: &Vector3) -> f32 {
        Self::directivity(
            (axial_direction.cross(target).norm()).atan2(axial_direction.dot(target)) * rad,
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) struct TestDirectivity {}

    impl Directivity for TestDirectivity {
        fn directivity(t: Angle) -> f32 {
            t.degree()
        }
    }

    #[rstest::rstest]
    #[test]
    #[case::dir_x(90., Vector3::x(), Vector3::z_axis())]
    #[case::dir_y(90., Vector3::y(), Vector3::z_axis())]
    #[case::dir_z(0., Vector3::z(), Vector3::z_axis())]
    fn test_directivity_from_dir(
        #[case] expected: f32,
        #[case] target: Vector3,
        #[case] dir: UnitVector3,
    ) {
        approx::assert_abs_diff_eq!(
            expected,
            TestDirectivity::directivity_from_dir(&dir, &target)
        );
    }
}
