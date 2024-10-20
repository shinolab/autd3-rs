use super::*;

#[allow(clippy::excessive_precision, clippy::unreadable_literal)]
static DIR_COEF_A: &[f32] = &[
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
static DIR_COEF_B: &[f32] = &[
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
static DIR_COEF_C: &[f32] = &[
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
static DIR_COEF_D: &[f32] = &[
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

pub struct T4010A1 {}

impl Directivity for T4010A1 {
    fn directivity(theta: Angle) -> f32 {
        let theta_deg = theta.degree().abs() % 180.0;
        let theta_deg = if theta_deg > 90.0 {
            180.0 - theta_deg
        } else {
            theta_deg
        };
        let i = (theta_deg / 10.0).ceil() as usize;
        if i == 0 {
            1.0
        } else {
            let x = theta_deg - (i as f32 - 1.0) * 10.0;
            ((DIR_COEF_D[i - 1] * x + DIR_COEF_C[i - 1]) * x + DIR_COEF_B[i - 1]) * x
                + DIR_COEF_A[i - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::defined::deg;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::deg_0(1.0, 0.0)]
    #[case::deg_10(1.0, 10.0)]
    #[case::deg_20(1.0, 20.0)]
    #[case::deg_30(0.891251, 30.0)]
    #[case::deg_40(0.70794576, 40.0)]
    #[case::deg_50(0.5011872, 50.0)]
    #[case::deg_60(0.35481337, 60.0)]
    #[case::deg_70(0.25118864, 70.0)]
    #[case::deg_80(0.19952622, 80.0)]
    #[case::deg_90(0.17783181, 90.0)]
    #[case::deg_100(0.19952622, 100.0)]
    fn test_directivity(#[case] expected: f32, #[case] theta_deg: f32) {
        approx::assert_abs_diff_eq!(expected, T4010A1::directivity(theta_deg * deg));
    }
}
