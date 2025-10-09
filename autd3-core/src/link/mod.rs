mod buffer_pool;
mod datagram;
mod error;

pub use buffer_pool::*;
pub use datagram::*;
pub use error::LinkError;

use crate::geometry::Geometry;

/// A trait that provides the interface with the device.
pub trait Link: Send {
    /// Opens the link.
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError>;

    /// Closes the link.
    fn close(&mut self) -> Result<(), LinkError>;

    #[doc(hidden)]
    fn update(&mut self, _: &Geometry) -> Result<(), LinkError> {
        Ok(())
    }

    /// Allocate a sending buffer for the link.
    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError>;

    /// Sends a message to the device.
    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError>;

    /// Receives a message from the device.
    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError>;

    /// Checks if the link is open.
    #[must_use]
    fn is_open(&self) -> bool;

    /// Ensures that the link is open, returning an error if it is not.
    fn ensure_is_open(&self) -> Result<(), LinkError> {
        if self.is_open() {
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }
}

impl Link for Box<dyn Link> {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        self.as_mut().open(geometry)
    }

    fn close(&mut self) -> Result<(), LinkError> {
        self.as_mut().close()
    }

    fn update(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        self.as_mut().update(geometry)
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        self.as_mut().alloc_tx_buffer()
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        self.as_mut().send(tx)
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.as_mut().receive(rx)
    }

    fn is_open(&self) -> bool {
        self.as_ref().is_open()
    }
}
