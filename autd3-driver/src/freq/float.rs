use super::{kHz, Freq, Frequency, Hz};

impl Frequency for Freq<f64> {}

impl Freq<f64> {
    pub const fn hz(&self) -> f64 {
        self.freq
    }
}

impl std::fmt::Display for Freq<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} Hz", self.freq)
    }
}

impl std::ops::Mul<Hz> for f64 {
    type Output = Freq<f64>;

    fn mul(self, _rhs: Hz) -> Self::Output {
        Self::Output { freq: self }
    }
}

impl std::ops::Mul<kHz> for f64 {
    type Output = Freq<f64>;

    fn mul(self, _rhs: kHz) -> Self::Output {
        Self::Output { freq: self * 1e3 }
    }
}
