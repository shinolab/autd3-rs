use std::num::NonZeroU8;

use autd3_derive::Builder;

use crate::firmware::operation::SilencerFixedUpdateRateOp;
use crate::{datagram::*, firmware::operation::SilencerTarget};

#[derive(Debug, Clone, Copy, Builder)]
pub struct FixedUpdateRate {
    #[get]
    pub(super) update_rate_intensity: NonZeroU8,
    #[get]
    pub(super) update_rate_phase: NonZeroU8,
    #[get]
    pub(super) target: SilencerTarget,
}

impl Silencer<FixedUpdateRate> {
    pub const fn with_target(mut self, target: SilencerTarget) -> Self {
        self.internal.target = target;
        self
    }
}

impl Silencer<FixedUpdateRate> {
    pub fn is_valid<T: WithSampling>(&self, _target: &T) -> bool {
        true
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gain::tests::TestGain;
    use modulation::tests::TestModulation;

    use crate::{derive::LoopBehavior, geometry::Vector3};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(FociSTM::new(
        SamplingConfig::FREQ_4K,
        [Vector3::zeros(), Vector3::zeros()]
    ).unwrap())]
    #[case(GainSTM::new(
        SamplingConfig::FREQ_4K,
        [TestGain{ data: Default::default(), err: None }, TestGain{ data: Default::default(), err: None }]
    ).unwrap())]
    #[case(TestModulation {
        buf: Arc::new(Vec::new()),
        config: SamplingConfig::FREQ_4K,
        loop_behavior: LoopBehavior::infinite(),
    })]
    #[cfg_attr(miri, ignore)]
    fn is_valid(#[case] target: impl WithSampling) {
        let s = Silencer::from_update_rate(NonZeroU8::new(1).unwrap(), NonZeroU8::new(1).unwrap());
        assert!(s.is_valid(&target));
    }
}
