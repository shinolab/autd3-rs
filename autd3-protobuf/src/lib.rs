#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod pb {
    tonic::include_proto!("autd3");
}

mod error;
#[cfg(feature = "lightweight")]
pub mod lightweight;
mod traits;

pub use error::*;
pub use traits::*;

pub use pb::*;
