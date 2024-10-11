use std::{f64::consts::PI, num::NonZeroUsize};

use super::InterpolationWindow;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blackman {
    pub size: NonZeroUsize,
}

impl InterpolationWindow for Blackman {
    fn value(&self, k: usize) -> f64 {
        let x = k as f64 / self.window_size() as f64;
        0.42 - 0.5 * (2.0 * PI * x).cos() + 0.08 * (4.0 * PI * x).cos()
    }

    fn window_size(&self) -> usize {
        self.size.get()
    }
}
