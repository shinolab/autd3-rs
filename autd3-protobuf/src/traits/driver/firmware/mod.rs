mod cpu;
#[cfg(feature = "lightweight")]
mod fpga;
#[cfg(feature = "lightweight")]
mod version;

#[cfg(feature = "lightweight")]
pub use fpga::to_transition_mode;
