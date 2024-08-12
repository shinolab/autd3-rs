mod float;
mod int;

pub struct Hz;
#[allow(non_camel_case_types)]
pub struct kHz;

pub trait Frequency:
    Clone + Copy + Sync + std::fmt::Debug + std::fmt::Display + PartialEq + PartialOrd
{
}

use derive_more::{Add, Display, Div, Mul, Sub};

#[derive(Clone, Copy, PartialEq, PartialOrd, Add, Div, Mul, Sub, Display)]
#[display("{} Hz", freq)]
pub struct Freq<T: Copy> {
    pub(crate) freq: T,
}

impl<T: Copy> Freq<T> {
    #[inline]
    pub const fn hz(&self) -> T {
        self.freq
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
    #[cfg_attr(miri, ignore)]
    fn display() {
        assert_eq!(format!("{}", 100 * Hz), "100 Hz");
        assert_eq!(format!("{}", 100 * kHz), "100000 Hz");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn dbg() {
        assert_eq!(format!("{:?}", 100 * Hz), "100 Hz");
        assert_eq!(format!("{:?}", 100 * kHz), "100000 Hz");
    }
}
