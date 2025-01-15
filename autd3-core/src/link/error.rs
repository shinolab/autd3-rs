use derive_more::Display;
use derive_new::new;
use thiserror::Error;

#[derive(new, Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
/// An error produced by the link.
pub struct LinkError {
    msg: String,
}
