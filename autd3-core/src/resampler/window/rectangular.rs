use std::num::NonZeroUsize;

use super::InterpolationWindow;

/// Rectangular window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangular {
    /// Window size.
    pub size: NonZeroUsize,
}

impl InterpolationWindow for Rectangular {
    fn value(&self, _idx: usize) -> f64 {
        1.0
    }

    fn window_size(&self) -> usize {
        self.size.get()
    }
}
