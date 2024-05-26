mod sphere;
mod t4010a1;

use crate::geometry::Vector3;

pub use sphere::Sphere;
pub use t4010a1::T4010A1;

pub trait Directivity: Send + Sync {
    fn directivity(theta_deg: f64) -> f64;
    fn directivity_from_dir(axial_direction: &Vector3, target: &Vector3) -> f64 {
        Self::directivity(
            (axial_direction.cross(target).norm())
                .atan2(axial_direction.dot(target))
                .to_degrees(),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct TestDirectivity {}

    impl Directivity for TestDirectivity {
        fn directivity(t: f64) -> f64 {
            t
        }
    }

    #[rstest::rstest]
    #[test]
    #[case::dir_x(90., Vector3::x(), Vector3::z())]
    #[case::dir_y(90., Vector3::y(), Vector3::z())]
    #[case::dir_z(0., Vector3::z(), Vector3::z())]
    fn test_directivity_from_dir(
        #[case] expected: f64,
        #[case] target: Vector3,
        #[case] dir: Vector3,
    ) {
        assert_approx_eq::assert_approx_eq!(
            expected,
            TestDirectivity::directivity_from_dir(&dir, &target)
        );
    }
}
