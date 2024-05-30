use crate::{
    defined::{Freq, Hz},
    error::AUTDInternalError,
    firmware::fpga::SamplingConfig,
    utils::float::is_integer,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum STMSamplingConfig {
    Freq(Freq<f32>),
    FreqNearest(Freq<f32>),
    SamplingConfig(SamplingConfig),
}

impl STMSamplingConfig {
    pub fn sampling(&self, size: usize) -> Result<SamplingConfig, AUTDInternalError> {
        match *self {
            STMSamplingConfig::Freq(f) => {
                let fs = f * size as f32;
                if !is_integer(fs.hz()) {
                    return Err(AUTDInternalError::STMFreqInvalid(size, f));
                }
                Ok(SamplingConfig::Freq(fs.hz() as u32 * Hz))
            }
            STMSamplingConfig::FreqNearest(f) => {
                Ok(SamplingConfig::FreqNearest(f.hz() * size as f32 * Hz))
            }
            Self::SamplingConfig(s) => Ok(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        defined::Hz,
        derive::{AUTDInternalError, SAMPLING_FREQ_DIV_MIN},
        firmware::fpga::{SamplingConfig, SAMPLING_FREQ_DIV_MAX},
    };

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::Freq(4000*Hz)), 4000.*Hz, 1)]
    #[case(Ok(SamplingConfig::Freq(8000*Hz)), 4000.*Hz, 2)]
    #[case(Ok(SamplingConfig::Freq(40000*Hz)), 40000.*Hz, 1)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(1, 4000.5*Hz)), 4000.5*Hz, 1)]
    fn frequency(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfig::Freq(freq).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::FreqNearest(4000.*Hz)), 4000.*Hz, 1)]
    #[case(Ok(SamplingConfig::FreqNearest(8000.*Hz)), 4000.*Hz, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(4001.*Hz)), 4001.*Hz, 1)]
    #[case(Ok(SamplingConfig::FreqNearest(40000.*Hz)), 40000.*Hz, 1)]
    fn frequency_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
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
