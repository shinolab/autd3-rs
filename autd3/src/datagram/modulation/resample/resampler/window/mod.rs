mod rectangular;
mod blackman;

pub use rectangular::Rectangular;
pub use blackman::Blackman;

pub trait InterpolationWindow {
    fn window_size(&self) -> usize;
    fn value(&self, k: usize) -> f64;
}
