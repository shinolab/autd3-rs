use super::*;

/// Directivity of spherical wave
pub struct Sphere {}

impl Directivity for Sphere {
    fn directivity(_: f64) -> f64 {
        1.
    }

    fn directivity_from_tr(_: &Transducer, _: &Vector3) -> f64 {
        1.
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::{Quaternion, UnitQuaternion};

    use super::*;

    use rand::prelude::*;

    #[test]
    fn test_directivity() {
        let mut rng = rand::thread_rng();
        assert_eq!(1.0, Sphere::directivity(rng.gen()));
    }

    #[rstest::fixture]
    fn tr() -> Transducer {
        let mut rng = rand::thread_rng();
        Transducer::new(
            rng.gen::<u8>() as usize,
            Vector3::new(rng.gen(), rng.gen(), rng.gen()),
            UnitQuaternion::from_quaternion(Quaternion::new(
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
            )),
        )
    }

    #[rstest::rstest]
    #[test]
    #[case::dir_x(1., Vector3::x())]
    #[case::dir_y(1., Vector3::y())]
    #[case::dir_z(1., Vector3::z())]
    fn test_directivity_sphere_from_tr(
        #[case] expected: f64,
        #[case] target: Vector3,
        tr: Transducer,
    ) {
        assert_eq!(expected, Sphere::directivity_from_tr(&tr, &target));
    }
}
