use crate::geometry::Geometry;

use super::{error::LinkError, RxMessage, TxMessage};

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

    /// Sends a message to the device.
    fn send(&mut self, tx: &[TxMessage]) -> Result<bool, LinkError>;

    /// Receives a message from the device.
    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError>;

    /// Checks if the link is open.
    #[must_use]
    fn is_open(&self) -> bool;
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

    fn send(&mut self, tx: &[TxMessage]) -> Result<bool, LinkError> {
        self.as_mut().send(tx)
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        self.as_mut().receive(rx)
    }

    fn is_open(&self) -> bool {
        self.as_ref().is_open()
    }
}
