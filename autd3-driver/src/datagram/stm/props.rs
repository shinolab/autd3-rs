use crate::{derive::*, firmware::fpga::STMSamplingConfiguration};

#[doc(hidden)]
#[derive(Clone, Copy, Builder)]
pub struct STMProps {
    pub(crate) config: STMSamplingConfiguration,
    #[getset]
    pub(crate) loop_behavior: LoopBehavior,
}

impl STMProps {
    pub const fn from_freq(freq: f64) -> Self {
        Self {
            config: STMSamplingConfiguration::Frequency(freq),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub const fn from_freq_nearest(freq: f64) -> Self {
        Self {
            config: STMSamplingConfiguration::FrequencyNearest(freq),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub const fn from_sampling_config(sampling: SamplingConfiguration) -> Self {
        Self {
            config: STMSamplingConfiguration::SamplingConfiguration(sampling),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub fn sampling_config(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        self.config.sampling(size)
    }
}
