use super::InterpolationWindow;

#[derive(Debug)]
pub struct Rectangular {
    pub size: usize,
}

impl InterpolationWindow for Rectangular {
    fn value(&self, _k: usize) -> f64 {
        1.0
    }

    fn window_size(&self) -> usize {
        self.size
    }
}
