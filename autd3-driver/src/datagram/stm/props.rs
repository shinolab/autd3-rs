use crate::{derive::*, firmware::fpga::STMSamplingConfig};

#[doc(hidden)]
#[derive(Clone, Copy, Builder)]
pub struct STMProps {
    pub(crate) config: STMSamplingConfig,
    #[getset]
    pub(crate) loop_behavior: LoopBehavior,
}

impl STMProps {
    pub const fn from_freq(freq: f64) -> Self {
        Self {
            config: STMSamplingConfig::Freq(freq),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub const fn from_freq_nearest(freq: f64) -> Self {
        Self {
            config: STMSamplingConfig::FreqNearest(freq),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub const fn from_sampling_config(sampling: SamplingConfig) -> Self {
        Self {
            config: STMSamplingConfig::SamplingConfig(sampling),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub fn sampling_config(&self, size: usize) -> Result<SamplingConfig, AUTDInternalError> {
        self.config.sampling(size)
    }
}
