use derive_more::Display;
use derive_new::new;
use thiserror::Error;

#[derive(new, Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
/// An error occurred during gain calculation.
pub struct GainError {
    msg: String,
}
