mod float;
mod int;

pub struct Hz;
#[allow(non_camel_case_types)]
pub struct kHz;

pub trait Frequency: Clone + Copy + Sync + std::fmt::Debug + std::fmt::Display + PartialEq {}

use derive_more::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, PartialEq, Add, Div, Mul, Sub)]
pub struct Freq<T: Copy> {
    pub(crate) freq: T,
}

impl<T: Copy> Freq<T> {
    #[inline]
    pub const fn hz(&self) -> T {
        self.freq
    }
}

impl<T: std::fmt::Display + Copy> std::fmt::Display for Freq<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} Hz", self.freq)
    }
}

impl<T: std::fmt::Debug + Copy> std::fmt::Debug for Freq<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} Hz", self.freq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(format!("{}", 100 * Hz), "100 Hz");
        assert_eq!(format!("{}", 100 * kHz), "100000 Hz");
    }

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", 100 * Hz), "100 Hz");
        assert_eq!(format!("{:?}", 100 * kHz), "100000 Hz");
    }
}
