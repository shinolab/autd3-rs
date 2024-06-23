use autd3_driver::defined::ABSOLUTE_THRESHOLD_OF_HEARING;

use derive_more::{Display, Div, Mul};

#[allow(non_camel_case_types)]
pub struct dB;
pub struct Pa;
#[allow(non_camel_case_types)]
pub struct kPa;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Div, Mul, Display)]
#[display(fmt = "{:.2} Pa", value)]
pub struct Amplitude {
    pub(crate) value: f32,
}

impl Amplitude {
    pub const fn pascal(&self) -> f32 {
        self.value
    }

    pub fn spl(&self) -> f32 {
        20.0 * f32::log10(self.value / ABSOLUTE_THRESHOLD_OF_HEARING)
    }
}

impl std::ops::Mul<dB> for f32 {
    type Output = Amplitude;

    fn mul(self, _rhs: dB) -> Self::Output {
        Self::Output {
            value: ABSOLUTE_THRESHOLD_OF_HEARING * f32::powf(10.0, self / 20.0),
        }
    }
}

impl std::ops::Mul<Pa> for f32 {
    type Output = Amplitude;

    fn mul(self, _rhs: Pa) -> Self::Output {
        Self::Output { value: self }
    }
}

impl std::ops::Mul<kPa> for f32 {
    type Output = Amplitude;

    fn mul(self, _rhs: kPa) -> Self::Output {
        Self::Output { value: self * 1e3 }
    }
}

impl std::ops::Mul<Amplitude> for f32 {
    type Output = Amplitude;

    fn mul(self, rhs: Amplitude) -> Self::Output {
        Self::Output {
            value: self * rhs.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db() {
        let amp = 121.5 * dB;

        assert_approx_eq::assert_approx_eq!(amp.spl(), 121.5, 1e-3);
        assert_approx_eq::assert_approx_eq!(amp.pascal(), 23.77, 1e-3);
    }

    #[test]
    fn test_pascal() {
        let amp = 23.77 * Pa;

        assert_approx_eq::assert_approx_eq!(amp.pascal(), 23.77, 1e-3);
        assert_approx_eq::assert_approx_eq!(amp.spl(), 121.5, 1e-3);

        assert_approx_eq::assert_approx_eq!((2. * amp).pascal(), 2. * 23.77, 1e-3);
        assert_approx_eq::assert_approx_eq!((amp * 2.).pascal(), 2. * 23.77, 1e-3);

        assert_approx_eq::assert_approx_eq!((amp / 2.).pascal(), 23.77 / 2., 1e-3);
    }

    #[test]
    fn test_kilo_pascal() {
        let amp = 23.77e-3 * kPa;

        assert_approx_eq::assert_approx_eq!(amp.pascal(), 23.77, 1e-3);
        assert_approx_eq::assert_approx_eq!(amp.spl(), 121.5, 1e-3);

        assert_approx_eq::assert_approx_eq!((2. * amp).pascal(), 2. * 23.77, 1e-3);
        assert_approx_eq::assert_approx_eq!((amp * 2.).pascal(), 2. * 23.77, 1e-3);

        assert_approx_eq::assert_approx_eq!((amp / 2.).pascal(), 23.77 / 2., 1e-3);
    }

    #[test]
    fn display() {
        let amp = 23.77 * Pa;
        assert_eq!(amp.to_string(), "23.77 Pa");
    }
}
