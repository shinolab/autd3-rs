#[cfg_attr(docsrs, doc(cfg(not(feature = "lightweight"))))]
#[cfg(not(feature = "lightweight"))]
mod autd3;
#[cfg_attr(docsrs, doc(cfg(not(feature = "lightweight"))))]
#[cfg(not(feature = "lightweight"))]
pub use autd3::*;

#[cfg_attr(docsrs, doc(cfg(feature = "lightweight")))]
#[cfg(feature = "lightweight")]
#[allow(clippy::large_enum_variant)]
mod lightweight;
#[cfg_attr(docsrs, doc(cfg(feature = "lightweight")))]
#[cfg(feature = "lightweight")]
pub use lightweight::*;
