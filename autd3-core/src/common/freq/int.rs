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
