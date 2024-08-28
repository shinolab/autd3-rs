use std::f64::consts::PI;

use super::InterpolationWindow;

pub struct Blackman {
    pub size: usize,
}

impl InterpolationWindow for Blackman {
    fn value(&self, k: usize) -> f64 {
        let x = k as f64 / self.size as f64;
        0.42 - 0.5 * (2.0 * PI * x).cos() + 0.08 * (4.0 * PI * x).cos()
    }

    fn window_size(&self) -> usize {
        self.size
    }
}
