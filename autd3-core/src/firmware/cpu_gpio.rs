#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuGPIOPort {
    pub pa5: bool,
    pub pa7: bool,
}

impl CpuGPIOPort {
    /// Creates a new [`CpuGPIOPort`].
    #[must_use]
    pub const fn new(pa5: bool, pa7: bool) -> Self {
        Self { pa5, pa7 }
    }
}
