#[cfg_attr(docsrs, doc(cfg(not(feature = "lightweight"))))]
#[cfg(not(feature = "lightweight"))]
mod autd3;
#[cfg_attr(docsrs, doc(cfg(not(feature = "lightweight"))))]
#[cfg(not(feature = "lightweight"))]
pub use autd3::*;

#[cfg_attr(docsrs, doc(cfg(feature = "lightweight")))]
#[cfg(feature = "lightweight")]
mod lightweight;
#[cfg_attr(docsrs, doc(cfg(feature = "lightweight")))]
#[cfg(feature = "lightweight")]
pub use lightweight::*;
