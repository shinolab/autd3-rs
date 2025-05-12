#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
mod r#async;
mod buffer_pool;
mod datagram;
mod error;
mod sync;

#[cfg(feature = "async")]
#[doc(inline)]
pub use r#async::*;
pub use buffer_pool::*;
pub use datagram::*;
pub use error::LinkError;
#[doc(inline)]
pub use sync::*;
