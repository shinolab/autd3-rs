#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides `Wav`, `RawPCM`, and `Csv` modulation.

mod csv;
mod error;
mod rawpcm;
mod wav;

pub use csv::Csv;
pub use rawpcm::RawPCM;
pub use wav::Wav;
