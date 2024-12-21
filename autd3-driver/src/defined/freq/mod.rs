mod float;
mod int;

pub struct Hz;
#[allow(non_camel_case_types)]
pub struct kHz;

use derive_more::{Add, Debug, Div, Mul, Sub};

#[derive(Clone, Copy, PartialEq, PartialOrd, Add, Div, Mul, Sub, Debug)]
#[debug("{} Hz", freq)]
pub struct Freq<T: Copy> {
    pub(crate) freq: T,
}

impl<T: Copy> Freq<T> {
    #[inline]
    pub const fn hz(&self) -> T {
        self.freq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", 100 * Hz), "100 Hz");
        assert_eq!(format!("{:?}", 100 * kHz), "100000 Hz");
    }
}
