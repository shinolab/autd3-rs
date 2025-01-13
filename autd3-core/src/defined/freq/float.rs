use super::{kHz, Freq, Hz};

impl std::ops::Mul<Hz> for f32 {
    type Output = Freq<f32>;

    fn mul(self, _rhs: Hz) -> Self::Output {
        Self::Output { freq: self }
    }
}

impl std::ops::Mul<kHz> for f32 {
    type Output = Freq<f32>;

    fn mul(self, _rhs: kHz) -> Self::Output {
        Self::Output { freq: self * 1e3 }
    }
}
