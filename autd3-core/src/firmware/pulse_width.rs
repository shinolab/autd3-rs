use crate::firmware::ULTRASOUND_PERIOD_COUNT_BITS;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
enum PulseWidthInner {
    Duty(f32),
    Raw(u32),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
/// The pulse width.
pub struct PulseWidth {
    inner: PulseWidthInner,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Error type for [`PulseWidth`].
pub enum PulseWidthError {
    /// Error when the pulse width is out of range.
    PulseWidthOutOfRange(u32, u32),
    /// Error when the duty ratio is out of range.
    DutyRatioOutOfRange(f32),
}

// GRCOV_EXCL_START
impl core::fmt::Display for PulseWidthError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PulseWidthError::PulseWidthOutOfRange(pulse_width, period) => {
                write!(
                    f,
                    "Pulse width ({}) is out of range [0, {})",
                    pulse_width, period
                )
            }
            PulseWidthError::DutyRatioOutOfRange(duty) => {
                write!(f, "Duty ratio ({}) is out of range [0, 1)", duty)
            }
        }
    }
}

impl core::error::Error for PulseWidthError {}
// GRCOV_EXCL_STOP

impl PulseWidth {
    /// Creates a new [`PulseWidth`].
    ///
    /// Note that the period depends on the firmware version, so it is recommended to use [`PulseWidth::from_duty`] instead.
    #[must_use]
    pub const fn new(pulse_width: u32) -> Self {
        Self {
            inner: PulseWidthInner::Raw(pulse_width),
        }
    }

    /// Creates a new [`PulseWidth`] from duty ratio.
    pub fn from_duty(duty: f32) -> Result<Self, PulseWidthError> {
        if !(0.0..1.0).contains(&duty) {
            return Err(PulseWidthError::DutyRatioOutOfRange(duty));
        };
        Ok(Self {
            inner: PulseWidthInner::Duty(duty),
        })
    }

    /// Returns the pulse width.
    pub fn pulse_width<T>(self) -> Result<T, PulseWidthError>
    where
        T: TryFrom<u32> + TryInto<u32>,
    {
        const PERIOD: u32 = 1 << ULTRASOUND_PERIOD_COUNT_BITS;
        let pulse_width = match self.inner {
            PulseWidthInner::Duty(duty) => (duty * PERIOD as f32).round() as u32,
            PulseWidthInner::Raw(raw) => raw,
        };
        if pulse_width >= PERIOD {
            return Err(PulseWidthError::PulseWidthOutOfRange(pulse_width, PERIOD));
        }
        T::try_from(pulse_width)
            .map_err(|_| PulseWidthError::PulseWidthOutOfRange(pulse_width, PERIOD))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case(Ok(0), 0)]
    #[case(Ok(256), 256)]
    #[case(Ok(511), 511)]
    #[case(Err(PulseWidthError::PulseWidthOutOfRange(512, 512)), 512)]
    #[test]
    fn test_pulse_width_new(
        #[case] expected: Result<u16, PulseWidthError>,
        #[case] pulse_width: u32,
    ) {
        assert_eq!(expected, PulseWidth::new(pulse_width).pulse_width());
    }

    #[rstest::rstest]
    #[case(Ok(0), 0.0)]
    #[case(Ok(256), 0.5)]
    #[case(Ok(511), 511.0 / 512.0)]
    #[case(Err(PulseWidthError::DutyRatioOutOfRange(-0.5)), -0.5)]
    #[case(Err(PulseWidthError::DutyRatioOutOfRange(1.0)), 1.0)]
    #[case(Err(PulseWidthError::DutyRatioOutOfRange(1.5)), 1.5)]
    #[test]
    fn test_pulse_width_from_duty(
        #[case] expected: Result<u16, PulseWidthError>,
        #[case] duty: f32,
    ) {
        assert_eq!(
            expected,
            PulseWidth::from_duty(duty).and_then(|p| p.pulse_width())
        );
    }
}
