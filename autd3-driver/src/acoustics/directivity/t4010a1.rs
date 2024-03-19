use super::*;

#[allow(clippy::excessive_precision, clippy::unreadable_literal)]
static DIR_COEF_A: &[float] = &[
    1.0,
    1.0,
    1.0,
    0.891250938,
    0.707945784,
    0.501187234,
    0.354813389,
    0.251188643,
    0.199526231,
];

#[allow(clippy::excessive_precision, clippy::unreadable_literal)]
static DIR_COEF_B: &[float] = &[
    0.,
    0.,
    -0.00459648054721,
    -0.0155520765675,
    -0.0208114779827,
    -0.0182211227016,
    -0.0122437497109,
    -0.00780345575475,
    -0.00312857467007,
];

#[allow(clippy::excessive_precision, clippy::unreadable_literal)]
static DIR_COEF_C: &[float] = &[
    0.,
    0.,
    -0.000787968093807,
    -0.000307591508224,
    -0.000218348633296,
    0.00047738416141,
    0.000120353137658,
    0.000323676257958,
    0.000143850511,
];

#[allow(clippy::excessive_precision, clippy::unreadable_literal)]
static DIR_COEF_D: &[float] = &[
    0.,
    0.,
    1.60125528528e-05,
    2.9747624976e-06,
    2.31910931569e-05,
    -1.1901034125e-05,
    6.77743734332e-06,
    -5.99548024824e-06,
    -4.79372835035e-06,
];

/// Directivity of T4010A1
pub struct T4010A1 {}

impl Directivity for T4010A1 {
    fn directivity(theta_deg: float) -> float {
        let theta_deg = theta_deg.abs() % 180.0;
        let theta_deg = if theta_deg > 90.0 {
            180.0 - theta_deg
        } else {
            theta_deg
        };
        let i = (theta_deg / 10.0).ceil() as usize;
        if i == 0 {
            1.0
        } else {
            let x = theta_deg - (i as float - 1.0) * 10.0;
            ((DIR_COEF_D[i - 1] * x + DIR_COEF_C[i - 1]) * x + DIR_COEF_B[i - 1]) * x
                + DIR_COEF_A[i - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::deg_0(1.0, 0.0)]
    #[case::deg_10(1.0, 10.0)]
    #[case::deg_20(1.0, 20.0)]
    #[case::deg_30(0.891251, 30.0)]
    #[case::deg_40(0.707946, 40.0)]
    #[case::deg_50(0.501187, 50.0)]
    #[case::deg_60(0.354813, 60.0)]
    #[case::deg_70(0.251189, 70.0)]
    #[case::deg_80(0.199526, 80.0)]
    #[case::deg_90(0.177831, 90.0)]
    fn test_directivity(#[case] expected: float, #[case] theta_deg: float) {
        assert_approx_eq!(expected, T4010A1::directivity(theta_deg));
    }
}
