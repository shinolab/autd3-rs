use crate::{
    geometry::Geometry,
    link::{LinkError, RxMessage, TxMessage},
};

/// A trait that provides the interface with the device.
pub trait AsyncLink: Send {
    /// Opens the link.
    fn open(
        &mut self,
        geometry: &Geometry,
    ) -> impl std::future::Future<Output = Result<(), LinkError>>;

    /// Closes the link.
    fn close(&mut self) -> impl std::future::Future<Output = Result<(), LinkError>>;

    #[doc(hidden)]
    fn update(&mut self, _: &Geometry) -> impl std::future::Future<Output = Result<(), LinkError>> {
        async { Ok(()) }
    }

    /// Allocate a sending buffer for the link.
    fn alloc_tx_buffer(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Vec<TxMessage>, LinkError>>;

    /// Sends a message to the device.
    fn send(
        &mut self,
        tx: Vec<TxMessage>,
    ) -> impl std::future::Future<Output = Result<(), LinkError>>;

    /// Receives a message from the device.
    fn receive(
        &mut self,
        rx: &mut [RxMessage],
    ) -> impl std::future::Future<Output = Result<(), LinkError>>;

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
