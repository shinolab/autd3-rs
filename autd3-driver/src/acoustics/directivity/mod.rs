mod sphere;
mod t4010a1;

use crate::geometry::{Transducer, Vector3};

pub use sphere::Sphere;
pub use t4010a1::T4010A1;

pub trait Directivity: Send + Sync {
    fn directivity(theta_deg: f64) -> f64;
    fn directivity_from_tr(tr: &Transducer, target: &Vector3) -> f64 {
        let dir = tr.axial_direction();
        Self::directivity(
            (dir.cross(target).norm())
                .atan2(dir.dot(target))
                .to_degrees(),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::geometry::UnitQuaternion;

    pub struct TestDirectivity {}

    impl Directivity for TestDirectivity {
        fn directivity(t: f64) -> f64 {
            t
        }
    }

    #[rstest::fixture]
    fn tr() -> Transducer {
        Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity())
    }

    #[rstest::rstest]
    #[test]
    #[case::dir_x(90., Vector3::x())]
    #[case::dir_y(90., Vector3::y())]
    #[cfg_attr(not(feature = "left_handed"), case::dir_z(0., Vector3::z()))]
    #[cfg_attr(feature = "left_handed", case::dir_z(0., -Vector3::z()))]
    fn test_directivity_from_tr(#[case] expected: f64, #[case] target: Vector3, tr: Transducer) {
        assert_approx_eq::assert_approx_eq!(
            expected,
            TestDirectivity::directivity_from_tr(&tr, &target)
        );
    }
}
