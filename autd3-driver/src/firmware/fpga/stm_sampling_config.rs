use crate::{error::AUTDInternalError, firmware::fpga::SamplingConfig, utils::float::is_integer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum STMSamplingConfig {
    Freq(f64),
    FreqNearest(f64),
    SamplingConfig(SamplingConfig),
}

impl STMSamplingConfig {
    pub fn sampling(&self, size: usize) -> Result<SamplingConfig, AUTDInternalError> {
        match *self {
            STMSamplingConfig::Freq(f) => {
                let fs = f * size as f64;
                if !is_integer(fs) {
                    return Err(AUTDInternalError::STMFreqInvalid(size, f));
                }
                Ok(SamplingConfig::Freq(fs as u32))
            }
            STMSamplingConfig::FreqNearest(f) => Ok(SamplingConfig::FreqNearest(f * size as f64)),
            Self::SamplingConfig(s) => Ok(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::STMSamplingConfig;
    use crate::{
        derive::{AUTDInternalError, SAMPLING_FREQ_DIV_MIN},
        firmware::fpga::{SamplingConfig, SAMPLING_FREQ_DIV_MAX},
    };

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::Freq(4000)), 4000., 1)]
    #[case(Ok(SamplingConfig::Freq(8000)), 4000., 2)]
    #[case(Ok(SamplingConfig::Freq(40000)), 40000., 1)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(1, 4000.5)), 4000.5, 1)]
    fn frequency(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: f64,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfig::Freq(freq).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::FreqNearest(4000.)), 4000., 1)]
    #[case(Ok(SamplingConfig::FreqNearest(8000.)), 4000., 2)]
    #[case(Ok(SamplingConfig::FreqNearest(4001.)), 4001., 1)]
    #[case(Ok(SamplingConfig::FreqNearest(40000.)), 40000., 1)]
    fn frequency_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: f64,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfig::FreqNearest(freq).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MIN), 1)]
    #[case(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MIN), 2)]
    #[case(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX), 1)]
    #[case(SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX), 2)]
    fn sampling(#[case] config: SamplingConfig, #[case] size: usize) {
        assert_eq!(
            Ok(config),
            STMSamplingConfig::SamplingConfig(config).sampling(size)
        );
    }
}
