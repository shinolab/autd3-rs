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

    fn ensure_is_open(&self) -> Result<(), LinkError> {
        self.as_ref().ensure_is_open()
    }
}

#[doc(hidden)]
pub trait AsyncLink: Link {
    fn open(
        &mut self,
        geometry: &Geometry,
    ) -> impl std::future::Future<Output = Result<(), LinkError>> {
        async { <Self as Link>::open(self, geometry) }
    }

    fn close(&mut self) -> impl std::future::Future<Output = Result<(), LinkError>> {
        async { <Self as Link>::close(self) }
    }

    fn update(
        &mut self,
        geometry: &Geometry,
    ) -> impl std::future::Future<Output = Result<(), LinkError>> {
        async { <Self as Link>::update(self, geometry) }
    }

    fn alloc_tx_buffer(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Vec<TxMessage>, LinkError>> {
        async { <Self as Link>::alloc_tx_buffer(self) }
    }

    fn send(
        &mut self,
        tx: Vec<TxMessage>,
    ) -> impl std::future::Future<Output = Result<(), LinkError>> {
        async { <Self as Link>::send(self, tx) }
    }

    fn receive(
        &mut self,
        rx: &mut [RxMessage],
    ) -> impl std::future::Future<Output = Result<(), LinkError>> {
        async { <Self as Link>::receive(self, rx) }
    }

    #[must_use]
    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }

    fn ensure_is_open(&self) -> Result<(), LinkError> {
        <Self as Link>::ensure_is_open(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLink {
        is_open: bool,
        open_called: bool,
        close_called: bool,
        update_called: bool,
        alloc_called: bool,
        send_called: bool,
        receive_called: bool,
    }

    impl MockLink {
        fn new() -> Self {
            Self {
                is_open: false,
                open_called: false,
                close_called: false,
                update_called: false,
                alloc_called: false,
                send_called: false,
                receive_called: false,
            }
        }
    }

    impl Link for MockLink {
        fn open(&mut self, _geometry: &Geometry) -> Result<(), LinkError> {
            self.open_called = true;
            self.is_open = true;
            Ok(())
        }

        fn close(&mut self) -> Result<(), LinkError> {
            self.close_called = true;
            self.is_open = false;
            Ok(())
        }

        fn update(&mut self, _geometry: &Geometry) -> Result<(), LinkError> {
            self.update_called = true;
            Ok(())
        }

        fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
            self.alloc_called = true;
            Ok(vec![])
        }

        fn send(&mut self, _tx: Vec<TxMessage>) -> Result<(), LinkError> {
            self.send_called = true;
            Ok(())
        }

        fn receive(&mut self, _rx: &mut [RxMessage]) -> Result<(), LinkError> {
            self.receive_called = true;
            Ok(())
        }

        fn is_open(&self) -> bool {
            self.is_open
        }
    }

    impl AsyncLink for MockLink {}

    #[test]
    fn open() -> Result<(), LinkError> {
        let mut link = MockLink::new();
        let geometry = Geometry::new(vec![]);

        assert!(!AsyncLink::is_open(&link));
        futures_lite::future::block_on(AsyncLink::open(&mut link, &geometry))?;
        assert!(link.open_called);
        assert!(AsyncLink::is_open(&link));

        Ok(())
    }

    #[test]
    fn close() -> Result<(), LinkError> {
        let mut link = MockLink::new();
        link.is_open = true;

        futures_lite::future::block_on(AsyncLink::close(&mut link))?;
        assert!(link.close_called);
        assert!(!AsyncLink::is_open(&link));

        Ok(())
    }

    #[test]
    fn update() -> Result<(), LinkError> {
        let mut link = MockLink::new();
        let geometry = Geometry::new(vec![]);

        futures_lite::future::block_on(AsyncLink::update(&mut link, &geometry))?;
        assert!(link.update_called);

        Ok(())
    }

    #[test]
    fn alloc_tx_buffer() -> Result<(), LinkError> {
        let mut link = MockLink::new();

        futures_lite::future::block_on(AsyncLink::alloc_tx_buffer(&mut link))?;
        assert!(link.alloc_called);

        Ok(())
    }

    #[test]
    fn send() -> Result<(), LinkError> {
        let mut link = MockLink::new();
        let tx = vec![];

        futures_lite::future::block_on(AsyncLink::send(&mut link, tx))?;
        assert!(link.send_called);

        Ok(())
    }

    #[test]
    fn receive() -> Result<(), LinkError> {
        let mut link = MockLink::new();
        let mut rx = vec![];

        futures_lite::future::block_on(AsyncLink::receive(&mut link, &mut rx))?;
        assert!(link.receive_called);

        Ok(())
    }

    #[test]
    fn is_open() {
        let mut link = MockLink::new();

        assert!(!AsyncLink::is_open(&link));
        link.is_open = true;
        assert!(AsyncLink::is_open(&link));
    }

    #[test]
    fn ensure_is_open() {
        let mut link = MockLink::new();

        let result = AsyncLink::ensure_is_open(&link);
        assert_eq!(Err(LinkError::closed()), result);
        link.is_open = true;
        assert!(AsyncLink::ensure_is_open(&link).is_ok());
    }
}
