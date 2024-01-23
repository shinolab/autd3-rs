mod error;

#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "remote")]
pub mod remote;

#[cfg(feature = "local")]
pub use local::TwinCAT;

#[cfg(feature = "remote")]
pub use remote::RemoteTwinCAT;
