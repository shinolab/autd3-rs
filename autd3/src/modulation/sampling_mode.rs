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
        assert_eq!(SamplingMode::ExactFrequency, SamplingMode::ExactFrequency);
        assert_eq!(SamplingMode::SizeOptimized, SamplingMode::SizeOptimized);
        assert_ne!(SamplingMode::ExactFrequency, SamplingMode::SizeOptimized);

        let mode = SamplingMode::SizeOptimized;
        let mode2 = mode;
        assert_eq!(mode, mode2);

        let mode = SamplingMode::SizeOptimized;
        assert_eq!(mode, mode.clone());

        assert_eq!(
            format!("{:?}", SamplingMode::ExactFrequency),
            "ExactFrequency"
        );
    }
}
