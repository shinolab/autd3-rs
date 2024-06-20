use std::time::Duration;

use crate::{datagram::*, firmware::operation::SilencerFixedCompletionStepsOp};

const NANOSEC: u128 = 1_000_000_000;

#[derive(Debug, Clone, Copy)]
pub struct FixedCompletionTime {
    pub(super) time_intensity: Duration,
    pub(super) time_phase: Duration,
    pub(super) strict_mode: bool,
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

    pub const fn completion_time_intensity(&self) -> Duration {
        self.internal.time_intensity
    }

    pub const fn completion_time_phase(&self) -> Duration {
        self.internal.time_phase
    }

    pub const fn strict_mode(&self) -> bool {
        self.internal.strict_mode
    }
}

pub struct SilencerFixedCompletionTimeOpGenerator {
    steps_intensity: u16,
    steps_phase: u16,
    strict_mode: bool,
}

impl OperationGenerator for SilencerFixedCompletionTimeOpGenerator {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self.steps_intensity, self.steps_phase, self.strict_mode),
            Self::O2::default(),
        )
    }
}

impl Datagram for Silencer<FixedCompletionTime> {
    type G = SilencerFixedCompletionTimeOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        let ultrasound_freq = geometry.ultrasound_freq().hz() as u128;
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
