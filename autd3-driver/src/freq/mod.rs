mod float;
mod int;

pub struct Hz;
#[allow(non_camel_case_types)]
pub struct kHz;

pub trait Frequency: Clone + Copy + Sync + std::fmt::Debug + std::fmt::Display + PartialEq {}

use derive_more::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Add, Div, Mul, Sub)]
pub struct Freq<T> {
    pub(crate) freq: T,
}
