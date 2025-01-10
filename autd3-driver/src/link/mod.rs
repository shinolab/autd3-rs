#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
mod r#async;
mod sync;

#[cfg(feature = "async")]
#[doc(inline)]
pub use r#async::*;
#[doc(inline)]
pub use sync::*;
