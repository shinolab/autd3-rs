const EPSILON: f32 = 1e-9;

pub fn is_integer(a: f32) -> bool {
    0.5 - (a.fract() - 0.5).abs() < EPSILON
}

#[cfg(test)]
mod tests {

    #[rstest::rstest]
    #[test]
    #[case(true, 1.0)]
    #[case(false, 1.5)]
    #[case(true, 1.0 + std::f32::EPSILON)]
    #[case(true, 1.0 - std::f32::EPSILON)]
    #[case(false, 1.0 + 1e-6)]
    #[case(false, 1.0 - 1e-6)]
    fn is_integer(#[case] expected: bool, #[case] a: f32) {
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
            let b = a as f32 / m as f32;
            assert!(super::is_integer(b * m as f32));
        });
    }
}
