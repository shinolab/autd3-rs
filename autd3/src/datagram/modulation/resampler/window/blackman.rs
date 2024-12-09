use std::{f64::consts::PI, num::NonZeroUsize};

use super::InterpolationWindow;

/// Blackman window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blackman {
    /// Window size.
    pub size: NonZeroUsize,
}

impl InterpolationWindow for Blackman {
    fn value(&self, idx: usize) -> f64 {
        let x = idx as f64 / self.window_size() as f64;
        0.42 - 0.5 * (2.0 * PI * x).cos() + 0.08 * (4.0 * PI * x).cos()
    }

    fn window_size(&self) -> usize {
        self.size.get()
    }
}
