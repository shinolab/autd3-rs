#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides `Wav`, and `Csv` modulation.

mod csv;
mod error;
mod wav;

pub use csv::{Csv, CsvOption};
pub use wav::Wav;
