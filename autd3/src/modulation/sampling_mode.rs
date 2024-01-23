#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplingMode {
    ExactFrequency,
    SizeOptimized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_mode_derive() {
        let mode = SamplingMode::ExactFrequency;
        assert_eq!(mode, SamplingMode::ExactFrequency);

        let mode = SamplingMode::SizeOptimized;
        assert_eq!(mode, SamplingMode::SizeOptimized);

        let mode = SamplingMode::ExactFrequency;
        assert_ne!(mode, SamplingMode::SizeOptimized);

        let mode = SamplingMode::SizeOptimized;
        let mode2 = mode;
        assert_eq!(mode, mode2);

        let mode = SamplingMode::SizeOptimized;
        let mode2 = mode.clone();
        assert_eq!(mode, mode2);

        assert_eq!(
            format!("{:?}", SamplingMode::ExactFrequency),
            "ExactFrequency"
        );
    }
}
