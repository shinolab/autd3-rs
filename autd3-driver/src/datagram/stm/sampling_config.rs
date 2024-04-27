use std::time::Duration;

use crate::{
    error::AUTDInternalError,
    firmware::fpga::{sampling_config, ultrasound_freq, SamplingConfiguration},
    utils::float::is_integer,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum STMSamplingConfiguration {
    Frequency(f64),
    FrequencyNearest(f64),
    Period(Duration),
    PeriodNearest(Duration),
    SamplingConfiguration(SamplingConfiguration),
}

impl STMSamplingConfiguration {
    pub fn sampling(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        match *self {
            STMSamplingConfiguration::Frequency(f) => {
                let fs = f * size as f64;
                if !is_integer(fs) {
                    return Err(AUTDInternalError::STMFrequencyInvalid(
                        size,
                        f,
                        ultrasound_freq() as f64 / size as f64,
                    ));
                }
                SamplingConfiguration::from_freq(fs as u32)
            }
            STMSamplingConfiguration::FrequencyNearest(f) => {
                SamplingConfiguration::from_freq_nearest(f * size as f64)
            }
            STMSamplingConfiguration::Period(p) => {
                if (p.as_nanos() % size as u128) != 0 {
                    return Err(AUTDInternalError::STMPeriodInvalid(
                        size,
                        p,
                        sampling_config::period_min() * size as u32,
                    ));
                }
                SamplingConfiguration::from_period(p / size as u32)
            }
            STMSamplingConfiguration::PeriodNearest(p) => {
                SamplingConfiguration::from_period_nearest(p / size as u32)
            }
            Self::SamplingConfiguration(s) => Ok(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::STMSamplingConfiguration;
    use crate::{
        derive::AUTDInternalError,
        firmware::fpga::{sampling_config, ultrasound_freq, SamplingConfiguration},
    };
    use std::time::Duration;

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_freq(4000).unwrap()), 4000., 1)]
    #[case(Ok(SamplingConfiguration::from_freq(8000).unwrap()), 4000., 2)]
    #[case(
        Err(AUTDInternalError::SamplingFreqInvalid(4001, ultrasound_freq())),
        4001.,
        1
    )]
    #[case(Ok(SamplingConfiguration::from_freq(sampling_config::freq_max()).unwrap()), sampling_config::freq_max() as _, 1)]
    #[case(
        Err(AUTDInternalError::SamplingFreqInvalid(sampling_config::freq_max() * 2, ultrasound_freq())),
        sampling_config::freq_max() as _,
        2
    )]
    fn frequency(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: f64,
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
    #[case(Ok(SamplingConfiguration::from_freq_nearest(sampling_config::freq_max_raw()).unwrap()), sampling_config::freq_max_raw(), 1)]
    #[case(
        Err(
            AUTDInternalError::SamplingFreqOutOfRange(
                sampling_config::freq_max_raw() * 2.,
                sampling_config::freq_min_raw(),
                sampling_config::freq_max_raw())
            ),
        sampling_config::freq_max_raw(),
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
    #[case(
        Err(AUTDInternalError::STMPeriodInvalid(
            3,
            Duration::from_micros(250),
            sampling_config::period_min() * 3
        )),
        Duration::from_micros(250),
        3
    )]
    #[case(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            Duration::from_micros(251),
            sampling_config::period_min()
        )),
        Duration::from_micros(251),
        1
    )]
    #[case(Ok(SamplingConfiguration::from_period(sampling_config::period_min()).unwrap()), sampling_config::period_min(), 1)]
    #[case(Err(AUTDInternalError::SamplingPeriodInvalid(sampling_config::period_min() / 2, sampling_config::period_min())), sampling_config::period_min(), 2)]
    #[case(Ok(SamplingConfiguration::from_period(sampling_config::period_max()).unwrap()), sampling_config::period_max(), 1)]
    #[case(
        Err(AUTDInternalError::SamplingPeriodInvalid(sampling_config::period_max() / 2, sampling_config::period_min())),
        sampling_config::period_max(),
        2
    )]
    fn period(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfiguration::Period(p).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(250)).unwrap()), Duration::from_micros(250), 1)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(125)).unwrap()), Duration::from_micros(250), 2)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(251)).unwrap()), Duration::from_micros(251), 1)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(sampling_config::period_min()).unwrap()), sampling_config::period_min(), 1)]
    #[case(Err(AUTDInternalError::SamplingPeriodOutOfRange(sampling_config::period_min() / 2, sampling_config::period_min_raw(), sampling_config::period_max_raw())), sampling_config::period_min(), 2)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(sampling_config::period_max()).unwrap()), sampling_config::period_max(), 1)]
    #[case(
        Ok(SamplingConfiguration::from_period_nearest(sampling_config::period_max() / 2).unwrap()),
        sampling_config::period_max(),
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
    #[case(SamplingConfiguration::from_division(sampling_config::div_min()).unwrap(), 1)]
    #[case(SamplingConfiguration::from_division(sampling_config::div_min()).unwrap(), 2)]
    #[case(SamplingConfiguration::from_division(sampling_config::div_max()).unwrap(), 1)]
    #[case(SamplingConfiguration::from_division(sampling_config::div_max()).unwrap(), 2)]
    #[case(SamplingConfiguration::from_period(sampling_config::period_min()).unwrap(), 1)]
    #[case(SamplingConfiguration::from_period(sampling_config::period_min()).unwrap(), 2)]
    #[case(SamplingConfiguration::from_period(sampling_config::period_max()).unwrap(), 1)]
    #[case(SamplingConfiguration::from_period(sampling_config::period_max()).unwrap(), 2)]
    #[case(SamplingConfiguration::from_freq(sampling_config::freq_min()).unwrap(), 1)]
    #[case(SamplingConfiguration::from_freq(sampling_config::freq_min()).unwrap(), 2)]
    #[case(SamplingConfiguration::from_freq(sampling_config::freq_max()).unwrap(), 1)]
    #[case(SamplingConfiguration::from_freq(sampling_config::freq_max()).unwrap(), 2)]
    fn sampling(#[case] config: SamplingConfiguration, #[case] size: usize) {
        assert_eq!(
            Ok(config),
            STMSamplingConfiguration::SamplingConfiguration(config).sampling(size)
        );
    }
}
