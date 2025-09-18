#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides [`Wav`], and [`Csv`] modulation.

#[cfg(feature = "csv")]
mod csv;
#[cfg(any(feature = "csv", feature = "wav"))]
mod error;
#[cfg(feature = "wav")]
mod wav;

#[cfg(feature = "csv")]
pub use csv::{Csv, CsvOption};
#[cfg(feature = "wav")]
pub use wav::Wav;
