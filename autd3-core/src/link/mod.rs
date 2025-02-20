#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
mod r#async;
mod datagram;
mod error;
mod sync;

#[cfg(feature = "async")]
#[doc(inline)]
pub use r#async::*;
pub use datagram::*;
pub use error::LinkError;
#[doc(inline)]
pub use sync::*;
