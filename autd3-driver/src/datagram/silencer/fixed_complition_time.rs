use std::time::Duration;

use crate::{
    datagram::*,
    defined::ULTRASOUND_FREQ,
    firmware::operation::{SilencerFixedCompletionStepsOp, Target},
};

const NANOSEC: u128 = 1_000_000_000;

#[derive(Debug, Clone, Copy)]
pub struct FixedCompletionTime {
    pub(super) time_intensity: Duration,
    pub(super) time_phase: Duration,
    pub(super) strict_mode: bool,
    pub(super) target: Target,
}

impl<T> std::ops::Mul<T> for FixedCompletionTime
where
    T: Copy,
    Duration: std::ops::Mul<T, Output = Duration>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            time_intensity: self.time_intensity * rhs,
            time_phase: self.time_phase * rhs,
            strict_mode: self.strict_mode,
            target: self.target,
        }
    }
}

impl<T> std::ops::Div<T> for FixedCompletionTime
where
    T: Copy,
    Duration: std::ops::Div<T, Output = Duration>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            time_intensity: self.time_intensity / rhs,
            time_phase: self.time_phase / rhs,
            strict_mode: self.strict_mode,
            target: self.target,
        }
    }
}

#[cfg(feature = "capi")]
impl Default for Silencer<FixedCompletionTime> {
    fn default() -> Self {
        Silencer {
            internal: FixedCompletionTime {
                time_intensity: Duration::ZERO,
                time_phase: Duration::ZERO,
                strict_mode: true,
            },
        }
    }
}

impl Silencer<FixedCompletionTime> {
    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.internal.strict_mode = strict_mode;
        self
    }

    pub const fn with_taget(mut self, target: Target) -> Self {
        self.internal.target = target;
        self
    }

    pub const fn completion_time_intensity(&self) -> Duration {
        self.internal.time_intensity
    }

    pub const fn completion_time_phase(&self) -> Duration {
        self.internal.time_phase
    }

    pub const fn strict_mode(&self) -> bool {
        self.internal.strict_mode
    }

    pub const fn target(&self) -> Target {
        self.internal.target
    }
}

#[derive(Debug)]
pub struct SilencerFixedCompletionTimeOpGenerator {
    steps_intensity: u16,
    steps_phase: u16,
    strict_mode: bool,
    target: Target,
}

impl OperationGenerator for SilencerFixedCompletionTimeOpGenerator {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.steps_intensity,
                self.steps_phase,
                self.strict_mode,
                self.target,
            ),
            Self::O2::default(),
        )
    }
}

impl Datagram for Silencer<FixedCompletionTime> {
    type G = SilencerFixedCompletionTimeOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        let ultrasound_freq = ULTRASOUND_FREQ.hz() as u128;
        let k_intensity = self.internal.time_intensity.as_nanos() * ultrasound_freq;
        let steps_intensity = if k_intensity % NANOSEC == 0 {
            (k_intensity / NANOSEC).min(u16::MAX as _)
        } else {
            return Err(AUTDInternalError::InvalidSilencerCompletionTime(
                self.internal.time_intensity,
            ));
        };

        let k_phase = self.internal.time_phase.as_nanos() * ultrasound_freq;
        let steps_phase = if k_phase % NANOSEC == 0 {
            (k_phase / NANOSEC).min(u16::MAX as _)
        } else {
            return Err(AUTDInternalError::InvalidSilencerCompletionTime(
                self.internal.time_phase,
            ));
        };

        Ok(SilencerFixedCompletionTimeOpGenerator {
            steps_intensity: steps_intensity as _,
            steps_phase: steps_phase as _,
            strict_mode: self.internal.strict_mode,
            target: self.internal.target,
        })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use crate::geometry::tests::create_geometry;

    use super::*;

    #[test]
    fn fixed_completion_time() {
        let d =
            Silencer::from_completion_time(Duration::from_micros(25), Duration::from_micros(50));
        assert_eq!(d.completion_time_intensity(), Duration::from_micros(25));
        assert_eq!(d.completion_time_phase(), Duration::from_micros(50));
        assert!(d.strict_mode());
    }

    #[test]
    fn invalid_time() {
        let geometry = create_geometry(1, 1);

        let d =
            Silencer::from_completion_time(Duration::from_micros(26), Duration::from_micros(50));

        assert_eq!(
            AUTDInternalError::InvalidSilencerCompletionTime(Duration::from_micros(26)),
            d.operation_generator(&geometry).unwrap_err()
        );

        let d =
            Silencer::from_completion_time(Duration::from_micros(25), Duration::from_micros(51));

        assert_eq!(
            AUTDInternalError::InvalidSilencerCompletionTime(Duration::from_micros(51)),
            d.operation_generator(&geometry).unwrap_err()
        );
    }
}
