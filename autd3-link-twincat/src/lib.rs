#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;

#[cfg(feature = "local")]
pub mod local;
#[cfg_attr(docsrs, doc(cfg(feature = "remote")))]
#[cfg(feature = "remote")]
pub mod remote;

#[cfg(feature = "local")]
pub use local::TwinCAT;

#[cfg_attr(docsrs, doc(cfg(feature = "remote")))]
#[cfg(feature = "remote")]
pub use remote::RemoteTwinCAT;
