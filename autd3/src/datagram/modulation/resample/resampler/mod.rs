mod sinc;
mod window;

use autd3_driver::{defined::Freq, derive::SamplingConfig, utils::float::is_integer};
pub use sinc::SincInterpolation;
pub use window::*;

pub trait Resampler {
    fn upsample(&self, buffer: &[u8], ratio: f64) -> Vec<u8>;
    fn downsample(&self, buffer: &[u8], ratio: f64) -> Vec<u8>;
    fn resample(&self, buffer: &[u8], source: Freq<f32>, target: SamplingConfig) -> Vec<u8> {
        let src_fs = source.hz().abs() as f64;
        let target_fs = target.freq().hz().abs() as f64;
        let ratio = target_fs / src_fs;
        if ratio > 1.0 {
            // GRCOV_EXCL_START
            if !is_integer(ratio) {
                tracing::warn!(
                    "Upsampling from {:?} to {:?} is not integer ratio",
                    source,
                    target.freq()
                );
            }
            // GRCOV_EXCL_STOP
            self.upsample(buffer, ratio)
        } else {
            // GRCOV_EXCL_START
            if !is_integer(src_fs / target_fs) {
                tracing::warn!(
                    "Downsampling from {:?} to {:?} is not integer ratio",
                    source,
                    target.freq()
                );
            }
            // GRCOV_EXCL_STOP
            self.downsample(buffer, ratio)
        }
    }
}
