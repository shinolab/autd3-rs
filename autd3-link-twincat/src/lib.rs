#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link to AUTD using TwinCAT3.

mod error;

#[cfg(feature = "local")]
/// Using TwinCAT3 on the local machine.
pub mod local;
#[cfg_attr(docsrs, doc(cfg(feature = "remote")))]
#[cfg(feature = "remote")]
/// Using TwinCAT3 on a remote machine.
pub mod remote;

#[cfg(feature = "local")]
pub use local::TwinCAT;

#[cfg_attr(docsrs, doc(cfg(feature = "remote")))]
#[cfg(feature = "remote")]
pub use remote::{AmsAddr, AmsNetId, RemoteTwinCAT, RemoteTwinCATOption, Source, Timeouts};
