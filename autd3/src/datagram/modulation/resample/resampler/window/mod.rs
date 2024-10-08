mod blackman;
mod rectangular;

pub use blackman::Blackman;
pub use rectangular::Rectangular;

pub trait InterpolationWindow: std::fmt::Debug + Clone + Copy + PartialEq + Send + Sync {
    fn window_size(&self) -> usize;
    fn value(&self, k: usize) -> f64;
}
