#[cfg_attr(docsrs, doc(cfg(feature = "link-audit")))]
#[cfg(feature = "link-audit")]
#[doc(hidden)]
pub mod audit;
#[cfg_attr(docsrs, doc(cfg(feature = "link-nop")))]
#[cfg(feature = "link-nop")]
mod nop;

#[cfg(feature = "link-audit")]
pub use audit::{Audit, AuditOption};
#[cfg(feature = "link-nop")]
pub use nop::Nop;
