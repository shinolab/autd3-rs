#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// The pulse width.
pub struct PulseWidth<const PERIOD: usize>(pub u16);

impl<const PERIOD: usize> PulseWidth<PERIOD> {
    /// Creates a new [`PulseWidth`] from duty ratio.
    #[must_use]
    pub fn from_duty(duty: f32) -> Self {
        let duty = if !(0.0..=1.0).contains(&duty) {
            tracing::warn!(
                "Duty ratio must be between 0 and 1, but got {}. Clamping to 0-1.",
                duty
            );
            duty.clamp(0.0, 1.0)
        } else {
            duty
        };
        Self((PERIOD as f32 * duty).round() as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERIOD: usize = 100;

    #[rstest::rstest]
    #[case(50, 0.5)]
    #[case(0, -0.5)]
    #[case(100, 1.5)]
    #[case(0, 0.0)]
    #[case(100, 1.0)]
    #[test]
    fn test_pulse_width_from_duty(#[case] expected: u16, #[case] duty: f32) {
        let pulse_width = PulseWidth::<PERIOD>::from_duty(duty);
        assert_eq!(expected, pulse_width.0);
    }
}
