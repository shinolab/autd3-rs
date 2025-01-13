use derive_more::derive::Display;
use thiserror::Error;

#[derive(Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
pub struct GainError {
    msg: String,
}
