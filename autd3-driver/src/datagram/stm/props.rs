use super::sampling_config::STMSamplingConfiguration;
use crate::{
    common::{LoopBehavior, SamplingConfiguration},
    derive::*,
    error::AUTDInternalError,
};

#[doc(hidden)]
#[derive(Clone, Copy, Builder)]
pub struct STMProps {
    sampling: STMSamplingConfiguration,
    #[getset]
    pub(crate) loop_behavior: LoopBehavior,
}

impl STMProps {
    pub const fn from_freq(freq: f64) -> Self {
        Self {
            sampling: STMSamplingConfiguration::Frequency(freq),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub const fn from_period(period: std::time::Duration) -> Self {
        Self {
            sampling: STMSamplingConfiguration::Period(period),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub const fn from_sampling_config(sampling: SamplingConfiguration) -> Self {
        Self {
            sampling: STMSamplingConfiguration::SamplingConfiguration(sampling),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub fn freq(&self, size: usize) -> f64 {
        self.sampling.frequency(size)
    }

    pub fn period(&self, size: usize) -> std::time::Duration {
        self.sampling.period(size)
    }

    pub fn sampling_config(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        self.sampling.sampling(size)
    }
}
