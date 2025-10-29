use super::{Freq, Hz, kHz};

impl core::ops::Mul<Hz> for f32 {
    type Output = Freq<f32>;

    fn mul(self, _rhs: Hz) -> Self::Output {
        Self::Output { freq: self }
    }
}

impl core::ops::Mul<kHz> for f32 {
    type Output = Freq<f32>;

    fn mul(self, _rhs: kHz) -> Self::Output {
        Self::Output { freq: self * 1e3 }
    }
}

impl core::ops::Mul<Freq<f32>> for f32 {
    type Output = Freq<f32>;

    fn mul(self, rhs: Freq<f32>) -> Self::Output {
        Self::Output {
            freq: self * rhs.freq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctor() {
        assert_eq!(Freq { freq: 200.0 }, 200.0 * Hz);
        assert_eq!(Freq { freq: 2000.0 }, 2.0 * kHz);
    }

    #[test]
    fn ops() {
        assert_eq!(200.0 * Hz, 2.0 * (100.0 * Hz));
    }
}
