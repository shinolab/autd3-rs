#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
mod r#async;
mod blocking;

#[cfg(feature = "async")]
pub use r#async::AsyncSender;
pub use blocking::Sender;
