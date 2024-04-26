use super::sampling_config::STMSamplingConfiguration;
use crate::derive::*;

#[doc(hidden)]
#[derive(Clone, Copy, Builder)]
pub struct STMProps {
    config: STMSamplingConfiguration,
    #[getset]
    pub(crate) loop_behavior: LoopBehavior,
}

impl STMProps {
    pub const fn from_freq(freq: u32) -> Self {
        Self {
            config: STMSamplingConfiguration::Frequency(freq),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub const fn from_freq_nearest(freq: f64) -> Self {
        Self {
            config: STMSamplingConfiguration::FrequencyNearest(freq),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub const fn from_period(period: std::time::Duration) -> Self {
        Self {
            config: STMSamplingConfiguration::Period(period),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub const fn from_period_nearest(period: std::time::Duration) -> Self {
        Self {
            config: STMSamplingConfiguration::PeriodNearest(period),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub const fn from_sampling_config(sampling: SamplingConfiguration) -> Self {
        Self {
            config: STMSamplingConfiguration::SamplingConfiguration(sampling),
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    pub fn sampling_config(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        self.config.sampling(size)
    }
}
