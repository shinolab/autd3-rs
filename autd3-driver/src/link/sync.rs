use std::time::Duration;

use crate::{
    derive::Geometry,
    error::AUTDDriverError,
    firmware::cpu::{RxMessage, TxMessage},
};

/// A trait that provides the interface with the device.
pub trait Link: Send {
    /// Closes the link.
    fn close(&mut self) -> Result<(), AUTDDriverError>;

    #[doc(hidden)]
    fn update(&mut self, _geometry: &Geometry) -> Result<(), AUTDDriverError> {
        Ok(())
    }

    /// Sends a message to the device.
    fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError>;

    /// Receives a message from the device.
    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError>;

    /// Checks if the link is open.
    #[must_use]
    fn is_open(&self) -> bool;

    #[doc(hidden)]
    fn trace(&mut self, _: Option<Duration>, _: Option<usize>) {}
}

/// A trait to build a link.
pub trait LinkBuilder: Send + Sync {
    /// The link type.
    type L: Link;

    /// Opens a link.
    fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDDriverError>;
}

impl Link for Box<dyn Link> {
    fn close(&mut self) -> Result<(), AUTDDriverError> {
        self.as_mut().close()
    }

    fn update(&mut self, geometry: &Geometry) -> Result<(), AUTDDriverError> {
        self.as_mut().update(geometry)
    }

    fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
        self.as_mut().send(tx)
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
        self.as_mut().receive(rx)
    }

    fn is_open(&self) -> bool {
        self.as_ref().is_open()
    }

    fn trace(&mut self, timeout: Option<Duration>, parallel_threshold: Option<usize>) {
        self.as_mut().trace(timeout, parallel_threshold)
    }
}
