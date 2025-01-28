use super::*;

/// Sphere directivity model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sphere {}

impl Directivity for Sphere {
    #[inline]
    fn directivity(_: Angle) -> f32 {
        1.
    }

    #[inline]
    fn directivity_from_dir(_: &UnitVector3, _: &Vector3) -> f32 {
        1.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::prelude::*;

    #[test]
    fn test_directivity() {
        let mut rng = rand::rng();
        assert_eq!(1.0, Sphere::directivity(rng.random::<f32>() * rad));
    }

    #[rstest::rstest]
    #[test]
    #[case::dir_x(1., Vector3::x())]
    #[case::dir_y(1., Vector3::y())]
    #[case::dir_z(1., Vector3::z())]
    fn test_directivity_sphere_from_dir(#[case] expected: f32, #[case] target: Vector3) {
        let mut rng = rand::rng();
        let dir =
            UnitVector3::new_unchecked(Vector3::new(rng.random(), rng.random(), rng.random()));
        assert_eq!(expected, Sphere::directivity_from_dir(&dir, &target));
    }
}
