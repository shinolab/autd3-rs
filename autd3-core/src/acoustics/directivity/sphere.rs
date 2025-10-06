use super::*;

/// Sphere directivity model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sphere {}

impl Directivity for Sphere {
    #[inline]
    fn directivity(_: Angle) -> f32 {
        1.
    }
}

#[cfg(test)]
mod tests {
    use crate::common::rad;

    use super::*;

    use rand::prelude::*;

    #[test]
    fn test_directivity() {
        let mut rng = rand::rng();
        assert_eq!(1.0, Sphere::directivity(rng.random::<f32>() * rad));
    }
}
