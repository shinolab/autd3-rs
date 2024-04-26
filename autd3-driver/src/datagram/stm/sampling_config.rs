use std::time::Duration;

use crate::{error::AUTDInternalError, firmware::fpga::SamplingConfiguration};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum STMSamplingConfiguration {
    Frequency(u32),
    FrequencyNearest(f64),
    Period(Duration),
    PeriodNearest(Duration),
    SamplingConfiguration(SamplingConfiguration),
}

impl STMSamplingConfiguration {
    pub fn sampling(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        match self {
            STMSamplingConfiguration::Frequency(f) => {
                SamplingConfiguration::from_freq(f * size as u32)
            }
            STMSamplingConfiguration::FrequencyNearest(f) => {
                SamplingConfiguration::from_freq_nearest(f * size as f64)
            }
            STMSamplingConfiguration::Period(p) => {
                SamplingConfiguration::from_period(*p / size as u32)
            }
            STMSamplingConfiguration::PeriodNearest(p) => {
                SamplingConfiguration::from_period_nearest(*p / size as u32)
            }
            Self::SamplingConfiguration(s) => Ok(*s),
        }
    }
}

#[cfg(test)]
mod tests {
    use Duration;

    use crate::firmware::fpga::ULTRASOUND_FREQUENCY;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_freq(4000).unwrap()), 4000, 1)]
    #[case(Ok(SamplingConfiguration::from_freq(8000).unwrap()), 4000, 2)]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(4001, ULTRASOUND_FREQUENCY)), 4001, 1)]
    #[case(Ok(SamplingConfiguration::from_freq(SamplingConfiguration::FREQ_MAX).unwrap()), SamplingConfiguration::FREQ_MAX, 1)]
    #[case(
        Err(AUTDInternalError::SamplingFreqInvalid(SamplingConfiguration::FREQ_MAX * 2, ULTRASOUND_FREQUENCY)),
        SamplingConfiguration::FREQ_MAX,
        2
    )]
    fn frequency(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: u32,
        #[case] size: usize,
    ) {
        assert_eq!(
            expect,
            STMSamplingConfiguration::Frequency(freq).sampling(size)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_freq_nearest(4000.).unwrap()), 4000., 1)]
    #[case(Ok(SamplingConfiguration::from_freq_nearest(8000.).unwrap()), 4000., 2)]
    #[case(Ok(SamplingConfiguration::from_freq_nearest(4001.).unwrap()), 4001., 1)]
    #[case(Ok(SamplingConfiguration::from_freq_nearest(SamplingConfiguration::FREQ_MAX_RAW).unwrap()), SamplingConfiguration::FREQ_MAX_RAW, 1)]
    #[case(
        Err(
            AUTDInternalError::SamplingFreqOutOfRange(
                SamplingConfiguration::FREQ_MAX_RAW * 2., 
                SamplingConfiguration::FREQ_MIN_RAW, 
                SamplingConfiguration::FREQ_MAX_RAW)
            ),  
        SamplingConfiguration::FREQ_MAX_RAW,
        2
    )]
    fn frequency_nearest(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: f64,
        #[case] size: usize,
    ) {
        assert_eq!(
            expect,
            STMSamplingConfiguration::FrequencyNearest(freq).sampling(size)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_period(Duration::from_micros(250)).unwrap()), Duration::from_micros(250), 1)]
    #[case(Ok(SamplingConfiguration::from_period(Duration::from_micros(125)).unwrap()), Duration::from_micros(250), 2)]
    #[case(Err(AUTDInternalError::SamplingPeriodInvalid(Duration::from_micros(251), SamplingConfiguration::PERIOD_MIN)), Duration::from_micros(251), 1)]
    #[case(Ok(SamplingConfiguration::from_period(SamplingConfiguration::PERIOD_MIN).unwrap()), SamplingConfiguration::PERIOD_MIN, 1)]
    #[case(Err(AUTDInternalError::SamplingPeriodInvalid(SamplingConfiguration::PERIOD_MIN / 2, SamplingConfiguration::PERIOD_MIN)), SamplingConfiguration::PERIOD_MIN, 2)]
    #[case(Ok(SamplingConfiguration::from_period(SamplingConfiguration::PERIOD_MAX).unwrap()), SamplingConfiguration::PERIOD_MAX, 1)]
    #[case(
        Err(AUTDInternalError::SamplingPeriodInvalid(SamplingConfiguration::PERIOD_MAX / 2, SamplingConfiguration::PERIOD_MIN)),
        SamplingConfiguration::PERIOD_MAX,
        2
    )]
    fn period(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(
            expect,
            STMSamplingConfiguration::Period(p).sampling(size)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(250)).unwrap()), Duration::from_micros(250), 1)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(125)).unwrap()), Duration::from_micros(250), 2)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(251)).unwrap()), Duration::from_micros(251), 1)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(SamplingConfiguration::PERIOD_MIN).unwrap()), SamplingConfiguration::PERIOD_MIN, 1)]
    #[case(Err(AUTDInternalError::SamplingPeriodOutOfRange(SamplingConfiguration::PERIOD_MIN / 2, SamplingConfiguration::PERIOD_MIN_RAW, SamplingConfiguration::PERIOD_MAX_RAW)), SamplingConfiguration::PERIOD_MIN, 2)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(SamplingConfiguration::PERIOD_MAX).unwrap()), SamplingConfiguration::PERIOD_MAX, 1)]
    #[case(
        Ok(SamplingConfiguration::from_period_nearest(SamplingConfiguration::PERIOD_MAX / 2).unwrap()),
        SamplingConfiguration::PERIOD_MAX,
        2
    )]
    fn period_nearest(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(
            expect,
            STMSamplingConfiguration::PeriodNearest(p).sampling(size)
        );
    }

    
    #[rstest::rstest]
    #[test]
    #[case(SamplingConfiguration::from_division(SamplingConfiguration::DIV_MIN).unwrap(), 1)]
    #[case(SamplingConfiguration::from_division(SamplingConfiguration::DIV_MIN).unwrap(), 2)]
    #[case(SamplingConfiguration::from_division(SamplingConfiguration::DIV_MAX).unwrap(), 1)]
    #[case(SamplingConfiguration::from_division(SamplingConfiguration::DIV_MAX).unwrap(), 2)]
    #[case(SamplingConfiguration::from_period(SamplingConfiguration::PERIOD_MIN).unwrap(), 1)]
    #[case(SamplingConfiguration::from_period(SamplingConfiguration::PERIOD_MIN).unwrap(), 2)]
    #[case(SamplingConfiguration::from_period(SamplingConfiguration::PERIOD_MAX).unwrap(), 1)]
    #[case(SamplingConfiguration::from_period(SamplingConfiguration::PERIOD_MAX).unwrap(), 2)]
    #[case(SamplingConfiguration::from_freq(SamplingConfiguration::FREQ_MIN).unwrap(), 1)]
    #[case(SamplingConfiguration::from_freq(SamplingConfiguration::FREQ_MIN).unwrap(), 2)]
    #[case(SamplingConfiguration::from_freq(SamplingConfiguration::FREQ_MAX).unwrap(), 1)]
    #[case(SamplingConfiguration::from_freq(SamplingConfiguration::FREQ_MAX).unwrap(), 2)]
    fn sampling_config(
        #[case] config: SamplingConfiguration,
        #[case] size: usize,
    ) {
        assert_eq!(
            Ok(config),
            STMSamplingConfiguration::SamplingConfiguration(config).sampling(size)
        );
    }
}
