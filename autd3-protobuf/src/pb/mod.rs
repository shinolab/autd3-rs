#[cfg(not(feature = "lightweight"))]
mod autd3;
#[cfg(not(feature = "lightweight"))]
pub use autd3::*;

#[cfg(feature = "lightweight")]
mod lightweight;
#[cfg(feature = "lightweight")]
pub use lightweight::*;
