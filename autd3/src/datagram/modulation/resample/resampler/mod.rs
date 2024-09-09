mod sinc;
mod window;

use autd3_driver::{defined::Freq, derive::SamplingConfig};
pub use sinc::SincInterpolation;
pub use window::*;

pub trait Resampler: std::fmt::Debug + Send + Sync {
    fn upsample(&self, buffer: &[u8], ratio: f64) -> Vec<u8>;
    fn downsample(&self, buffer: &[u8], ratio: f64) -> Vec<u8>;
    fn resample(&self, buffer: &[u8], source: Freq<f32>, target: SamplingConfig) -> Vec<u8> {
        let src_fs = source.hz().abs() as f64;
        let target_fs = target.freq().hz().abs() as f64;
        let ratio = target_fs / src_fs;
        if ratio > 1.0 {
            self.upsample(buffer, ratio)
        } else {
            self.downsample(buffer, ratio)
        }
    }
}
