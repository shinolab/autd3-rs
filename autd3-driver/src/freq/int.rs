use super::{kHz, Freq, Hz};
use derive_more::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Add, Div, Mul, Sub)]
pub struct FreqInt {
    freq: u32,
}

impl Freq for FreqInt {}

impl FreqInt {
    pub const fn hz(&self) -> u32 {
        self.freq
    }
}

impl std::fmt::Display for FreqInt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} Hz", self.freq)
    }
}

impl std::ops::Mul<Hz> for u32 {
    type Output = FreqInt;

    fn mul(self, _rhs: Hz) -> Self::Output {
        FreqInt { freq: self }
    }
}

impl std::ops::Mul<kHz> for u32 {
    type Output = FreqInt;

    fn mul(self, _rhs: kHz) -> Self::Output {
        FreqInt { freq: self * 1000 }
    }
}
