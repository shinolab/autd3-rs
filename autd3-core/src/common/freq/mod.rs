mod float;
mod int;

/// \[Hz\]
pub struct Hz;

/// \[kHz\]
#[allow(non_camel_case_types)]
pub struct kHz;

use derive_more::{Add, Debug, Div, Mul, Sub};

/// Frequency
#[derive(Clone, Copy, PartialEq, PartialOrd, Add, Div, Mul, Sub, Debug)]
#[debug("{} Hz", freq)]
pub struct Freq<T: Copy> {
    pub(crate) freq: T,
}

impl<T: Copy> Freq<T> {
    #[inline]
    /// Returns the frequency in Hz.
    pub const fn hz(&self) -> T {
        self.freq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbg() {
        assert_eq!(alloc::format!("{:?}", 100 * Hz), "100 Hz");
        assert_eq!(alloc::format!("{:?}", 100 * kHz), "100000 Hz");
    }
}
