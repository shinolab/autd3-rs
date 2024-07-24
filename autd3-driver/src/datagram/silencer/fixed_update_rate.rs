use std::num::NonZeroU8;

use crate::firmware::operation::SilencerFixedUpdateRateOp;
use crate::{datagram::*, firmware::operation::SilencerTarget};

#[derive(Debug, Clone, Copy)]
pub struct FixedUpdateRate {
    pub(super) update_rate_intensity: NonZeroU8,
    pub(super) update_rate_phase: NonZeroU8,
    pub(super) target: SilencerTarget,
}

impl Silencer<FixedUpdateRate> {
    pub const fn with_taget(mut self, target: SilencerTarget) -> Self {
        self.internal.target = target;
        self
    }

    pub const fn update_rate_intensity(&self) -> u8 {
        self.internal.update_rate_intensity.get()
    }

    pub const fn update_rate_phase(&self) -> u8 {
        self.internal.update_rate_phase.get()
    }

    pub const fn target(&self) -> SilencerTarget {
        self.internal.target
    }
}

pub struct SilencerFixedUpdateRateOpGenerator {
    update_rate_intensity: NonZeroU8,
    update_rate_phase: NonZeroU8,
    target: SilencerTarget,
}

impl OperationGenerator for SilencerFixedUpdateRateOpGenerator {
    type O1 = SilencerFixedUpdateRateOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.update_rate_intensity,
                self.update_rate_phase,
                self.target,
            ),
            Self::O2::default(),
        )
    }
}

impl Datagram for Silencer<FixedUpdateRate> {
    type G = SilencerFixedUpdateRateOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SilencerFixedUpdateRateOpGenerator {
            update_rate_intensity: self.internal.update_rate_intensity,
            update_rate_phase: self.internal.update_rate_phase,
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

#[cfg(feature = "capi")]
impl Default for Silencer<FixedUpdateRate> {
    fn default() -> Self {
        Silencer::from_update_rate(NonZeroU8::MIN, NonZeroU8::MIN)
    }
}
