use super::{kHz, Freq, Hz};
use derive_more::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Add, Div, Mul, Sub)]
pub struct FreqFloat {
    freq: f64,
}

impl Freq for FreqFloat {}

impl FreqFloat {
    pub const fn hz(&self) -> f64 {
        self.freq
    }
}

impl std::fmt::Display for FreqFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} Hz", self.freq)
    }
}

impl std::ops::Mul<Hz> for f64 {
    type Output = FreqFloat;

    fn mul(self, _rhs: Hz) -> Self::Output {
        FreqFloat { freq: self }
    }
}

impl std::ops::Mul<kHz> for f64 {
    type Output = FreqFloat;

    fn mul(self, _rhs: kHz) -> Self::Output {
        FreqFloat { freq: self * 1e3 }
    }
}
