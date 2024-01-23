use autd3_driver::defined::{float, ABSOLUTE_THRESHOLD_OF_HEARING};

#[allow(non_camel_case_types)]
pub struct dB;
pub struct Pascal;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Amplitude {
    // Amplitude in Pascal
    pub(crate) value: float,
}

impl Amplitude {
    pub const fn as_pascal(&self) -> float {
        self.value
    }

    pub fn as_spl(&self) -> float {
        20.0 * float::log10(self.value / ABSOLUTE_THRESHOLD_OF_HEARING)
    }
}

impl std::ops::Mul<dB> for float {
    type Output = Amplitude;

    fn mul(self, _rhs: dB) -> Self::Output {
        Self::Output {
            value: ABSOLUTE_THRESHOLD_OF_HEARING * float::powf(10.0, self / 20.0),
        }
    }
}

impl std::ops::Mul<Pascal> for float {
    type Output = Amplitude;

    fn mul(self, _rhs: Pascal) -> Self::Output {
        Self::Output { value: self }
    }
}

impl std::ops::Mul<Amplitude> for float {
    type Output = Amplitude;

    fn mul(self, rhs: Amplitude) -> Self::Output {
        Self::Output {
            value: self * rhs.value,
        }
    }
}

impl std::ops::Mul<float> for Amplitude {
    type Output = Amplitude;

    fn mul(self, rhs: float) -> Self::Output {
        Self::Output {
            value: self.value * rhs,
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
    }

    #[test]
    fn test_amp_derive() {
        let amp = 23.77 * Pascal;
        let amp2 = amp.clone();

        assert_eq!(amp, amp2);
        assert_eq!(format!("{:?}", amp), "Amplitude { value: 23.77 }");
        assert!(!(amp < amp2));
        assert!(!(amp > amp2));
        assert!(amp <= amp2);
        assert!(amp >= amp2);
        assert!(amp <= 2.0 * amp);
    }
}
