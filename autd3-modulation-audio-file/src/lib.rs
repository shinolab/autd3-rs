#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod error;
mod rawpcm;
mod wav;

pub use rawpcm::RawPCM;
pub use wav::Wav;
