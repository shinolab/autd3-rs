use crate::{error::AUTDInternalError, firmware::fpga::SamplingConfiguration, utils::float::is_integer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum STMSamplingConfiguration {
    Frequency(f64),
    FrequencyNearest(f64),
    SamplingConfiguration(SamplingConfiguration),
}

impl STMSamplingConfiguration {
    pub fn sampling(&self, size: usize) -> Result<SamplingConfiguration, AUTDInternalError> {
        match *self {
            STMSamplingConfiguration::Frequency(f) => {
                let fs = f * size as f64;
                if !is_integer(fs) {
                    return Err(AUTDInternalError::STMFrequencyInvalid(size, f));
                }
                Ok(SamplingConfiguration::Frequency(fs as u32))
            }
            STMSamplingConfiguration::FrequencyNearest(f) => {
                Ok(SamplingConfiguration::FrequencyNearest(f * size as f64))
            }
            Self::SamplingConfiguration(s) => Ok(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::STMSamplingConfiguration;
    use crate::{
        derive::{AUTDInternalError, SAMPLING_FREQ_DIV_MIN},
        firmware::fpga::{SamplingConfiguration, SAMPLING_FREQ_DIV_MAX},
    };

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::Frequency(4000)), 4000., 1)]
    #[case(Ok(SamplingConfiguration::Frequency(8000)), 4000., 2)]
    #[case(Ok(SamplingConfiguration::Frequency(40000)), 40000., 1)]
    #[case(Err(AUTDInternalError::STMFrequencyInvalid(1, 4000.5)), 4000.5, 1)]
    fn frequency(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: f64,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfiguration::Frequency(freq).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::FrequencyNearest(4000.)), 4000., 1)]
    #[case(Ok(SamplingConfiguration::FrequencyNearest(8000.)), 4000., 2)]
    #[case(Ok(SamplingConfiguration::FrequencyNearest(4001.)), 4001., 1)]
    #[case(Ok(SamplingConfiguration::FrequencyNearest(40000.)), 40000., 1)]
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
    #[case(SamplingConfiguration::DivisionRaw(SAMPLING_FREQ_DIV_MIN), 1)]
    #[case(SamplingConfiguration::DivisionRaw(SAMPLING_FREQ_DIV_MIN), 2)]
    #[case(SamplingConfiguration::DivisionRaw(SAMPLING_FREQ_DIV_MAX), 1)]
    #[case(SamplingConfiguration::DivisionRaw(SAMPLING_FREQ_DIV_MAX), 2)]
    fn sampling(#[case] config: SamplingConfiguration, #[case] size: usize) {
        assert_eq!(
            Ok(config),
            STMSamplingConfiguration::SamplingConfiguration(config).sampling(size)
        );
    }
}
