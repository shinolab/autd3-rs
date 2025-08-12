#[cfg(not(feature = "std"))]
use num_traits::float::Float;

const EPSILON: f64 = 1e-6;

#[doc(hidden)]
#[must_use]
pub fn is_integer(a: f64) -> bool {
    0.5 - (a.fract() - 0.5).abs() < EPSILON
}

#[cfg(test)]
mod tests {

    #[rstest::rstest]
    #[case(true, 1.0)]
    #[case(false, 1.5)]
    #[case(true, 1.0 + f32::EPSILON)]
    #[case(true, 1.0 - f32::EPSILON)]
    #[case(false, 1.0 + 1e-3)]
    #[case(false, 1.0 - 1e-3)]
    fn is_integer(#[case] expected: bool, #[case] a: f32) {
        assert_eq!(super::is_integer(a as f64), expected, "{a}");
    }

    #[test]
    fn is_integer_rand() {
        use rand::Rng;

        const N: usize = 100000;

        let mut rng = rand::rng();

        (0..N).for_each(|_| {
            let a: u32 = rng.random_range(1..40000);
            let m: u32 = rng.random_range(512..=0xFFFFFFFF);
            let b = a as f64 / m as f64;
            assert!(super::is_integer(b * m as f64));
        });
    }
}
