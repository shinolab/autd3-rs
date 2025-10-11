#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link to AUTD using [EtherCrab](https://github.com/ethercrab-rs/ethercrab).

mod error;
mod ethercrab_link;
mod inner;

#[cfg(feature = "core_affinity")]
pub use core_affinity;
pub use ethercrab::{MainDeviceConfig, RetryBehaviour, Timeouts, subdevice_group::DcConfiguration};
pub use ethercrab_link::EtherCrab;
pub use inner::{EtherCrabOption, EtherCrabOptionFull, Status};
pub use thread_priority;
