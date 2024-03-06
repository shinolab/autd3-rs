mod sphere;
mod t4010a1;

use crate::{
    defined::float,
    geometry::{Transducer, Vector3},
};

pub use sphere::Sphere;
pub use t4010a1::T4010A1;

/// Directivity
pub trait Directivity: Send + Sync {
    fn directivity(theta_deg: float) -> float;
    fn directivity_from_tr(tr: &Transducer, target: &Vector3) -> float {
        let dir = tr.z_direction();
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
        fn directivity(t: float) -> float {
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
    #[case::dir_z(0., Vector3::z())]
    fn test_directivity_from_tr(#[case] expected: float, #[case] target: Vector3, tr: Transducer) {
        assert_approx_eq::assert_approx_eq!(
            expected,
            TestDirectivity::directivity_from_tr(&tr, &target)
        );
    }
}
