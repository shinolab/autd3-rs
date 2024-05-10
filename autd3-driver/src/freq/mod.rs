mod float;
mod int;

pub use float::FreqFloat;
pub use int::FreqInt;

pub struct Hz;
#[allow(non_camel_case_types)]
pub struct kHz;

pub trait Freq: Clone + Copy + Sync + std::fmt::Debug + std::fmt::Display + PartialEq {}
