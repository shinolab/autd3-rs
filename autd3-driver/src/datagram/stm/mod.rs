mod focus;
mod gain;

pub use focus::{ChangeFocusSTMSegment, FocusSTM};
pub use gain::{ChangeGainSTMSegment, GainSTM};

use crate::{
    common::{LoopBehavior, SamplingConfiguration},
    defined::float,
    error::AUTDInternalError,
};

enum STMSamplingConfiguration {
    Frequency(float),
    Period(std::time::Duration),
    SamplingConfiguration(SamplingConfiguration),
}

impl STMSamplingConfiguration {
    pub fn frequency(&self, size: usize) -> float {
        match self {
            Self::Frequency(f) => *f,
            Self::Period(p) => 1000000000. / p.as_nanos() as float,
            Self::SamplingConfiguration(s) => s.frequency() / size as float,
        }
    }

    pub fn period(&self, size: usize) -> std::time::Duration {
        match self {
            Self::Frequency(f) => std::time::Duration::from_nanos((1000000000. / f) as _),
            Self::Period(p) => *p,
            Self::SamplingConfiguration(s) => s.period() * size as u32,
        }
    }

    pub fn sampling(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        match self {
            Self::Frequency(f) => {
                let min = SamplingConfiguration::FREQ_MIN / size as float;
                let max = SamplingConfiguration::FREQ_MAX / size as float;
                SamplingConfiguration::from_frequency(f * size as float)
                    .map_err(|_| AUTDInternalError::STMFreqOutOfRange(size, *f, min, max))
            }
            Self::Period(p) => {
                let min = SamplingConfiguration::PERIOD_MIN as usize / size;
                let max = SamplingConfiguration::PERIOD_MAX as usize / size;
                SamplingConfiguration::from_period(std::time::Duration::from_nanos(
                    (p.as_nanos() as usize / size) as _,
                ))
                .map_err(|_| AUTDInternalError::STMPeriodOutOfRange(size, p.as_nanos(), min, max))
            }
            Self::SamplingConfiguration(s) => Ok(*s),
        }
    }
}

#[doc(hidden)]
pub struct STMProps {
    sampling: STMSamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl STMProps {
    pub const fn from_freq(freq: float) -> Self {
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

    pub const fn with_loop_behavior(self, loop_behavior: LoopBehavior) -> Self {
        Self {
            loop_behavior,
            ..self
        }
    }

    pub const fn loop_behavior(&self) -> LoopBehavior {
        self.loop_behavior
    }

    pub fn freq(&self, size: usize) -> float {
        self.sampling.frequency(size)
    }

    pub fn period(&self, size: usize) -> std::time::Duration {
        self.sampling.period(size)
    }

    pub fn sampling_config(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        self.sampling.sampling(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency() {
        let config = STMSamplingConfiguration::Frequency(4e3);
        assert_eq!(config.frequency(1), 4e3);
        assert_eq!(config.frequency(2), 4e3);
        assert_eq!(config.period(1), std::time::Duration::from_micros(250));
        assert_eq!(config.period(2), std::time::Duration::from_micros(250));
        assert_eq!(
            config.sampling(1).unwrap(),
            SamplingConfiguration::from_frequency(4e3).unwrap()
        );
        assert_eq!(
            config.sampling(2).unwrap(),
            SamplingConfiguration::from_frequency(8e3).unwrap()
        );

        let config = STMSamplingConfiguration::Frequency(0.1);
        assert_eq!(config.frequency(65536), 0.1);
        assert_eq!(config.period(65536), std::time::Duration::from_secs(10));
        assert_eq!(
            config.sampling(65536).unwrap(),
            SamplingConfiguration::from_frequency(0.1 * 65536.0).unwrap()
        );
    }

    #[test]
    fn test_period() {
        let config = STMSamplingConfiguration::Period(std::time::Duration::from_micros(250));
        assert_eq!(config.frequency(1), 4e3);
        assert_eq!(config.frequency(2), 4e3);
        assert_eq!(config.period(1), std::time::Duration::from_micros(250));
        assert_eq!(config.period(2), std::time::Duration::from_micros(250));
        assert_eq!(
            config.sampling(1).unwrap(),
            SamplingConfiguration::from_frequency(4e3).unwrap()
        );
        assert_eq!(
            config.sampling(2).unwrap(),
            SamplingConfiguration::from_frequency(8e3).unwrap()
        );

        let config = STMSamplingConfiguration::Period(std::time::Duration::from_secs(10));
        assert_eq!(config.frequency(65536), 0.1);
        assert_eq!(config.period(65536), std::time::Duration::from_secs(10));
        assert_eq!(
            config.sampling(65536).unwrap(),
            SamplingConfiguration::from_period(std::time::Duration::from_nanos(
                10 * 1000 * 1000 * 1000 / 65536
            ))
            .unwrap()
        );
    }

    #[test]
    fn test_sampling() {
        let config = STMSamplingConfiguration::SamplingConfiguration(
            SamplingConfiguration::from_frequency(4e3).unwrap(),
        );
        assert_eq!(config.frequency(1), 4e3);
        assert_eq!(config.frequency(2), 2e3);
        assert_eq!(config.period(1), std::time::Duration::from_micros(250));
        assert_eq!(config.period(2), std::time::Duration::from_micros(500));
        assert_eq!(
            config.sampling(1).unwrap(),
            SamplingConfiguration::from_frequency(4e3).unwrap()
        );
        assert_eq!(
            config.sampling(2).unwrap(),
            SamplingConfiguration::from_frequency(4e3).unwrap()
        );
    }

    #[test]
    fn test_frequency_out_of_range() {
        let config = STMSamplingConfiguration::Frequency(40e3);
        assert_eq!(
            config.sampling(1),
            Ok(SamplingConfiguration::from_frequency(40e3).unwrap())
        );
        assert_eq!(
            config.sampling(2),
            Err(AUTDInternalError::STMFreqOutOfRange(
                2,
                40e3,
                SamplingConfiguration::FREQ_MIN / 2.,
                SamplingConfiguration::FREQ_MAX / 2.,
            ))
        );
    }

    #[test]
    fn test_period_out_of_range() {
        let config = STMSamplingConfiguration::Period(std::time::Duration::from_micros(25));
        assert_eq!(
            config.sampling(1),
            Ok(SamplingConfiguration::from_frequency(40e3).unwrap())
        );
        assert_eq!(
            config.sampling(2),
            Err(AUTDInternalError::STMPeriodOutOfRange(
                2,
                25000,
                SamplingConfiguration::PERIOD_MIN as usize / 2,
                SamplingConfiguration::PERIOD_MAX as usize / 2,
            ))
        );
    }
}
