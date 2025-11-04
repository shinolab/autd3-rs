mod float;
mod int;

/// \[Hz\]
pub struct Hz;

/// \[kHz\]
#[allow(non_camel_case_types)]
pub struct kHz;

/// Frequency
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Freq<T: Copy> {
    pub(crate) freq: T,
}

impl<T: Copy> core::fmt::Debug for Freq<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} Hz", self.freq)
    }
}

impl<T: Copy> Freq<T> {
    #[inline]
    /// Returns the frequency in Hz.
    pub const fn hz(&self) -> T {
        self.freq
    }
}

impl<T> core::ops::Add<Freq<T>> for Freq<T>
where
    T: core::ops::Add<Output = T> + Copy,
{
    type Output = Freq<T>;

    fn add(self, rhs: Freq<T>) -> Self::Output {
        Freq {
            freq: self.freq + rhs.freq,
        }
    }
}

impl<T> core::ops::Sub<Freq<T>> for Freq<T>
where
    T: core::ops::Sub<Output = T> + Copy,
{
    type Output = Freq<T>;

    fn sub(self, rhs: Freq<T>) -> Self::Output {
        Freq {
            freq: self.freq - rhs.freq,
        }
    }
}

impl<T, U> core::ops::Mul<U> for Freq<T>
where
    T: core::ops::Mul<U, Output = T> + Copy,
{
    type Output = Freq<T>;

    fn mul(self, rhs: U) -> Self::Output {
        Freq {
            freq: self.freq * rhs,
        }
    }
}

impl<T, U> core::ops::Div<U> for Freq<T>
where
    T: core::ops::Div<U, Output = T> + Copy,
{
    type Output = Freq<T>;

    fn div(self, rhs: U) -> Self::Output {
        Freq {
            freq: self.freq / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ops() {
        assert_eq!(200 * Hz, 100 * Hz + 100 * Hz);
        assert_eq!(0 * Hz, 100 * Hz - 100 * Hz);
        assert_eq!(200 * Hz, 100 * Hz * 2);
        assert_eq!(50 * Hz, 100 * Hz / 2);
    }

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", 100 * Hz), "100 Hz");
        assert_eq!(format!("{:?}", 100 * kHz), "100000 Hz");
    }
}
