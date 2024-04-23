use crate::{error::AUTDInternalError, firmware::fpga::SamplingConfiguration};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum STMSamplingConfiguration {
    Frequency(f64),
    Period(std::time::Duration),
    SamplingConfiguration(SamplingConfiguration),
}

impl STMSamplingConfiguration {
    pub fn frequency(&self, size: usize) -> f64 {
        match self {
            Self::Frequency(f) => *f,
            Self::Period(p) => 1000000000. / p.as_nanos() as f64,
            Self::SamplingConfiguration(s) => s.frequency() / size as f64,
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
                let min = SamplingConfiguration::FREQ_MIN / size as f64;
                let max = SamplingConfiguration::FREQ_MAX / size as f64;
                SamplingConfiguration::from_frequency(f * size as f64)
                    .map_err(|_| AUTDInternalError::STMFreqOutOfRange(size, *f as _, min, max))
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
