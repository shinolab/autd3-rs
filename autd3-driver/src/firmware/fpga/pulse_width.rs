use getset::CopyGetters;
use num::Zero;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, CopyGetters)]
/// The pulse width.
pub struct PulseWidth<T: Copy, const BITS: usize> {
    #[getset(get_copy = "pub")]
    /// The pulse width in period of 2^`BITS`.
    pulse_width: T,
}

impl<T, const BITS: usize> PulseWidth<T, BITS>
where
    T: Copy + TryFrom<usize> + Zero + PartialOrd,
{
    /// Creates a new [`PulseWidth`].
    pub fn new(pulse_width: T) -> Option<Self> {
        let period: T = T::try_from(1 << BITS).ok()?;
        if pulse_width < T::zero() || period <= pulse_width {
            return None;
        }
        Some(Self { pulse_width })
    }

    /// Creates a new [`PulseWidth`] from duty ratio.
    #[must_use]
    pub fn from_duty(duty: f32) -> Option<Self> {
        if !(0.0..=1.0).contains(&duty) {
            return None;
        } else {
            duty
        };
        Self::new(T::try_from(((1 << BITS) as f32 * duty).round() as usize).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BITS: usize = 9;

    #[rstest::rstest]
    #[case(Some(0), 0)]
    #[case(Some(256), 256)]
    #[case(Some(511), 511)]
    #[case(None, 512)]
    #[test]
    fn test_pulse_width_new(#[case] expected: Option<u16>, #[case] pulse_width: u16) {
        let pulse_width = PulseWidth::<u16, BITS>::new(pulse_width);
        assert_eq!(expected, pulse_width.map(|p| p.pulse_width()));
    }

    #[rstest::rstest]
    #[case(Some(0), 0.0)]
    #[case(Some(256), 0.5)]
    #[case(Some(511), 511.0 / 512.0)]
    #[case(None, -0.5)]
    #[case(None, 1.0)]
    #[case(None, 1.5)]
    #[test]
    fn test_pulse_width_from_duty(#[case] expected: Option<u16>, #[case] duty: f32) {
        let pulse_width = PulseWidth::<u16, BITS>::from_duty(duty);
        assert_eq!(expected, pulse_width.map(|p| p.pulse_width()));
    }
}
