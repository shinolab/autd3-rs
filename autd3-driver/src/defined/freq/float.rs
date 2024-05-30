use super::{kHz, Freq, Frequency, Hz};

impl Frequency for Freq<f32> {}

impl Freq<f32> {
    pub const fn hz(&self) -> f32 {
        self.freq
    }
}

impl std::fmt::Display for Freq<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} Hz", self.freq)
    }
}

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
