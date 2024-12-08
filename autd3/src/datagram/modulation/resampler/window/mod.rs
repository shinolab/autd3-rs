mod blackman;
mod rectangular;

pub use blackman::Blackman;
pub use rectangular::Rectangular;

/// Interpolation window trait.
pub trait InterpolationWindow: std::fmt::Debug + Clone + Copy + PartialEq + Send + Sync {
    /// Get the window size.
    fn window_size(&self) -> usize;
    /// Get the value of the window at given index.
    fn value(&self, idx: usize) -> f64;
}
