/// The parallel processing mode.
#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelMode {
    /// Automatically select the processing mode. If the number of devices is greater than the parallel threshold of the [`Datagram::option`], the parallel processing is used.
    #[default]
    Auto = 0,
    /// Force to use the parallel processing.
    On = 1,
    /// Force to use the serial processing.
    Off = 2,
}

impl ParallelMode {
    #[must_use]
    pub(crate) const fn is_parallel(self, num_devices: usize, parallel_threshold: usize) -> bool {
        match self {
            ParallelMode::On => true,
            ParallelMode::Off => false,
            ParallelMode::Auto => num_devices > parallel_threshold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case(true, ParallelMode::On, 1, 1)]
    #[case(true, ParallelMode::On, 2, 1)]
    #[case(true, ParallelMode::On, 1, 2)]
    #[case(false, ParallelMode::Off, 1, 1)]
    #[case(false, ParallelMode::Off, 2, 1)]
    #[case(false, ParallelMode::Off, 1, 2)]
    #[case(false, ParallelMode::Auto, 1, 1)]
    #[case(true, ParallelMode::Auto, 2, 1)]
    #[case(false, ParallelMode::Auto, 1, 2)]
    #[test]
    fn parallel_mode(
        #[case] expect: bool,
        #[case] mode: ParallelMode,
        #[case] num_devices: usize,
        #[case] threshold: usize,
    ) {
        assert_eq!(expect, mode.is_parallel(num_devices, threshold));
    }
}
