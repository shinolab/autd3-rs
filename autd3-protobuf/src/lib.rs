#[cfg(feature = "tonic-build")]
mod pb {
    tonic::include_proto!("autd3");
}

#[cfg(not(feature = "tonic-build"))]
mod pb;

mod error;
#[cfg(feature = "lightweight")]
pub mod lightweight;
mod traits;

pub use error::*;
pub use traits::*;

pub use pb::*;
