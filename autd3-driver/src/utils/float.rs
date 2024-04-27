const EPSILON: f64 = 1e-9;

pub fn is_integer(a: f64) -> bool {
    0.5 - (a.fract() - 0.5).abs() < EPSILON
}

#[cfg(test)]
mod tests {

    #[rstest::rstest]
    #[test]
    #[case(true, 1.0)]
    #[case(false, 1.5)]
    #[case(true, 1.0 + std::f64::EPSILON)]
    #[case(true, 1.0 - std::f64::EPSILON)]
    #[case(false, 1.0 + 1e-6)]
    #[case(false, 1.0 - 1e-6)]
    fn is_integer(#[case] expected: bool, #[case] a: f64) {
        assert_eq!(super::is_integer(a), expected);
    }

    #[test]
    fn is_integer_rand() {
        use rand::{thread_rng, Rng};

        const N: usize = 100000;

        let mut rng = thread_rng();

        (0..N).for_each(|_| {
            let a: u32 = rng.gen_range(1..40000);
            let m: u32 = rng.gen_range(512..=0xFFFFFFFF);
            let b = a as f64 / m as f64;
            assert!(super::is_integer(b * m as f64));
        });
    }
}
