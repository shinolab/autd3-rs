use super::{Freq, Hz, kHz};

impl std::ops::Mul<Hz> for u32 {
    type Output = Freq<u32>;

    fn mul(self, _rhs: Hz) -> Self::Output {
        Self::Output { freq: self }
    }
}

impl std::ops::Mul<kHz> for u32 {
    type Output = Freq<u32>;

    fn mul(self, _rhs: kHz) -> Self::Output {
        Self::Output { freq: self * 1000 }
    }
}
