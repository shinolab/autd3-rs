#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "local")]
pub use local::{EthernetAdapters, Status, SyncMode, SOEM};

#[cfg(feature = "remote")]
pub mod remote;
#[cfg(feature = "remote")]
pub use remote::RemoteSOEM;
