use crate::firmware::{
    fpga::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
    operation::SilencerFixedCompletionStepsOp,
};
use crate::{datagram::*, firmware::operation::SilencerTarget};

#[derive(Debug, Clone, Copy)]
pub struct FixedCompletionSteps {
    pub(super) steps_intensity: u16,
    pub(super) steps_phase: u16,
    pub(super) strict_mode: bool,
    pub(super) target: SilencerTarget,
}

impl<T> std::ops::Mul<T> for FixedCompletionSteps
where
    T: Copy,
    u16: std::ops::Mul<T, Output = u16>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            steps_intensity: self.steps_intensity * rhs,
            steps_phase: self.steps_phase * rhs,
            strict_mode: self.strict_mode,
            target: self.target,
        }
    }
}

impl<T> std::ops::Div<T> for FixedCompletionSteps
where
    T: Copy,
    u16: std::ops::Div<T, Output = u16>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            steps_intensity: self.steps_intensity / rhs,
            steps_phase: self.steps_phase / rhs,
            strict_mode: self.strict_mode,
            target: self.target,
        }
    }
}

impl Default for Silencer<FixedCompletionSteps> {
    fn default() -> Self {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity: SILENCER_STEPS_INTENSITY_DEFAULT,
                steps_phase: SILENCER_STEPS_PHASE_DEFAULT,
                strict_mode: true,
                target: SilencerTarget::Intensity,
            },
        }
    }
}

impl Silencer<FixedCompletionSteps> {
    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.internal.strict_mode = strict_mode;
        self
    }

    pub const fn with_taget(mut self, target: SilencerTarget) -> Self {
        self.internal.target = target;
        self
    }

    pub const fn completion_steps_intensity(&self) -> u16 {
        self.internal.steps_intensity
    }

    pub const fn completion_steps_phase(&self) -> u16 {
        self.internal.steps_phase
    }

    pub const fn strict_mode(&self) -> bool {
        self.internal.strict_mode
    }

    pub const fn target(&self) -> SilencerTarget {
        self.internal.target
    }
}

pub struct SilencerFixedCompletionStepsOpGenerator {
    steps_intensity: u16,
    steps_phase: u16,
    strict_mode: bool,
    target: SilencerTarget,
}

impl OperationGenerator for SilencerFixedCompletionStepsOpGenerator {
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

impl Datagram for Silencer<FixedCompletionSteps> {
    type G = SilencerFixedCompletionStepsOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SilencerFixedCompletionStepsOpGenerator {
            steps_intensity: self.internal.steps_intensity,
            steps_phase: self.internal.steps_phase,
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
