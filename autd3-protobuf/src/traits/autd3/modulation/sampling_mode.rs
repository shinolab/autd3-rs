use crate::pb::*;

impl From<SamplingMode> for autd3::modulation::SamplingMode {
    fn from(value: SamplingMode) -> Self {
        match value {
            SamplingMode::ExactFreq => Self::ExactFrequency,
            SamplingMode::SizeOpt => Self::SizeOptimized,
        }
    }
}

impl From<autd3::modulation::SamplingMode> for SamplingMode {
    fn from(value: autd3::modulation::SamplingMode) -> Self {
        match value {
            autd3::modulation::SamplingMode::ExactFrequency => Self::ExactFreq,
            autd3::modulation::SamplingMode::SizeOptimized => Self::SizeOpt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_mode() {
        let mode = autd3::modulation::SamplingMode::ExactFrequency;
        let msg: SamplingMode = mode.into();
        let mode2 = autd3::modulation::SamplingMode::from(msg);
        assert_eq!(mode, mode2);

        let mode = autd3::modulation::SamplingMode::SizeOptimized;
        let msg: SamplingMode = mode.into();
        let mode2 = autd3::modulation::SamplingMode::from(msg);
        assert_eq!(mode, mode2);
    }
}
