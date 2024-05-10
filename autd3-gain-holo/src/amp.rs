use autd3_driver::defined::ABSOLUTE_THRESHOLD_OF_HEARING;

#[allow(non_camel_case_types)]
pub struct dB;
pub struct Pascal;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Amplitude {
    // Amplitude in Pascal
    pub(crate) value: f64,
}

impl Amplitude {
    pub const fn as_pascal(&self) -> f64 {
        self.value
    }

    pub fn as_spl(&self) -> f64 {
        20.0 * f64::log10(self.value / ABSOLUTE_THRESHOLD_OF_HEARING)
    }
}

impl std::ops::Mul<dB> for f64 {
    type Output = Amplitude;

    fn mul(self, _rhs: dB) -> Self::Output {
        Self::Output {
            value: ABSOLUTE_THRESHOLD_OF_HEARING * f64::powf(10.0, self / 20.0),
        }
    }
}

impl std::ops::Mul<Pascal> for f64 {
    type Output = Amplitude;

    fn mul(self, _rhs: Pascal) -> Self::Output {
        Self::Output { value: self }
    }
}

impl std::ops::Mul<Amplitude> for f64 {
    type Output = Amplitude;

    fn mul(self, rhs: Amplitude) -> Self::Output {
        Self::Output {
            value: self * rhs.value,
        }
    }
}

impl std::ops::Mul<f64> for Amplitude {
    type Output = Amplitude;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            value: self.value * rhs,
        }
    }
}

impl std::ops::Div<f64> for Amplitude {
    type Output = Amplitude;

    fn div(self, rhs: f64) -> Self::Output {
        Self::Output {
            value: self.value / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db() {
        let amp = 121.5 * dB;

        assert_approx_eq::assert_approx_eq!(amp.as_spl(), 121.5, 1e-3);
        assert_approx_eq::assert_approx_eq!(amp.as_pascal(), 23.77, 1e-3);
    }

    #[test]
    fn test_pascal() {
        let amp = 23.77 * Pascal;

        assert_approx_eq::assert_approx_eq!(amp.as_pascal(), 23.77, 1e-3);
        assert_approx_eq::assert_approx_eq!(amp.as_spl(), 121.5, 1e-3);

        assert_approx_eq::assert_approx_eq!((2. * amp).as_pascal(), 2. * 23.77, 1e-3);
        assert_approx_eq::assert_approx_eq!((amp * 2.).as_pascal(), 2. * 23.77, 1e-3);

        assert_approx_eq::assert_approx_eq!((amp / 2.).as_pascal(), 23.77 / 2., 1e-3);
    }
}
