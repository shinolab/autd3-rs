#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
mod r#async;
mod datagram;
mod error;
mod sync;

pub use datagram::*;
pub use error::LinkError;
#[cfg(feature = "async")]
#[doc(inline)]
pub use r#async::*;
#[doc(inline)]
pub use sync::*;
