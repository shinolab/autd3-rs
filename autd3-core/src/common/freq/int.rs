use super::{Freq, Hz, kHz};

impl core::ops::Mul<Hz> for u32 {
    type Output = Freq<u32>;

    fn mul(self, _rhs: Hz) -> Self::Output {
        Self::Output { freq: self }
    }
}

impl core::ops::Mul<kHz> for u32 {
    type Output = Freq<u32>;

    fn mul(self, _rhs: kHz) -> Self::Output {
        Self::Output { freq: self * 1000 }
    }
}

impl core::ops::Mul<Freq<u32>> for u32 {
    type Output = Freq<u32>;

    fn mul(self, rhs: Freq<u32>) -> Self::Output {
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
        assert_eq!(Freq { freq: 200 }, 200 * Hz);
        assert_eq!(Freq { freq: 2000 }, 2 * kHz);
    }

    #[test]
    fn ops() {
        assert_eq!(200 * Hz, 2 * (100 * Hz));
    }
}
