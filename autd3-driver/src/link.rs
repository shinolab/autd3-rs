use std::time::Duration;

use crate::{
    cpu::{check_if_msg_is_processed, RxMessage, TxDatagram},
    error::AUTDInternalError,
    geometry::Geometry,
};

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    /// Link is a interface to the AUTD device
    #[async_trait::async_trait]
    pub trait Link: Send + Sync {
        /// Close link
        async fn close(&mut self) -> Result<(), AUTDInternalError>;
        /// Send data to devices
        async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError>;
        /// Receive data from devices
        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError>;
        /// Check if link is open
        #[must_use]
        fn is_open(&self) -> bool;
        /// Get timeout
        #[must_use]
        fn timeout(&self) -> Duration;
    }

    #[async_trait::async_trait]
    pub trait LinkBuilder: Send + Sync {
        type L: Link;

        /// Open link
        async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError>;
    }

    #[async_trait::async_trait]
    impl Link for Box<dyn Link> {
        async fn close(&mut self) -> Result<(), AUTDInternalError> {
            self.as_mut().close().await
        }

        async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
            self.as_mut().send(tx).await
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
            self.as_mut().receive(rx).await
        }

        fn is_open(&self) -> bool {
            self.as_ref().is_open()
        }

        fn timeout(&self) -> Duration {
            self.as_ref().timeout()
        }
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    /// Link is a interface to the AUTD device
    pub trait Link: Send + Sync {
        /// Close link
        fn close(&mut self) -> impl std::future::Future<Output = Result<(), AUTDInternalError>>;
        /// Send data to devices
        fn send(
            &mut self,
            tx: &TxDatagram,
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;
        /// Receive data from devices
        fn receive(
            &mut self,
            rx: &mut [RxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;
        /// Check if link is open
        #[must_use]
        fn is_open(&self) -> bool;
        /// Get timeout
        #[must_use]
        fn timeout(&self) -> Duration;
    }

    pub trait LinkBuilder {
        type L: Link;

        /// Open link
        fn open(
            self,
            geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<Self::L, AUTDInternalError>>;
    }
}

#[cfg(feature = "async-trait")]
pub use internal::Link;
#[cfg(feature = "async-trait")]
pub use internal::LinkBuilder;

#[cfg(not(feature = "async-trait"))]
pub use internal::Link;
#[cfg(not(feature = "async-trait"))]
pub use internal::LinkBuilder;

/// Send and receive data
pub async fn send_receive<L: Link>(
    link: &mut L,
    tx: &TxDatagram,
    rx: &mut [RxMessage],
    timeout: Option<Duration>,
) -> Result<bool, AUTDInternalError> {
    let timeout = timeout.unwrap_or(link.timeout());
    if !link.send(tx).await? {
        return Ok(false);
    }
    if timeout.is_zero() {
        return link.receive(rx).await;
    }
    wait_msg_processed(link, tx, rx, timeout).await
}

/// Wait until message is processed
pub async fn wait_msg_processed<L: Link>(
    link: &mut L,
    tx: &TxDatagram,
    rx: &mut [RxMessage],
    timeout: Duration,
) -> Result<bool, AUTDInternalError> {
    let start = std::time::Instant::now();
    loop {
        if link.receive(rx).await? && check_if_msg_is_processed(tx, rx).all(std::convert::identity)
        {
            return Ok(true);
        }
        if start.elapsed() > timeout {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
    rx.iter()
        .try_fold((), |_, r| Result::<(), AUTDInternalError>::from(r))
        .map(|_| false)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLink {
        pub is_open: bool,
        pub timeout: Duration,
        pub send_cnt: usize,
        pub recv_cnt: usize,
        pub down: bool,
    }

    impl Link for MockLink {
        async fn close(&mut self) -> Result<(), AUTDInternalError> {
            self.is_open = false;
            Ok(())
        }

        async fn send(&mut self, _: &TxDatagram) -> Result<bool, AUTDInternalError> {
            if !self.is_open {
                return Err(AUTDInternalError::LinkClosed);
            }

            self.send_cnt += 1;
            Ok(!self.down)
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
            if !self.is_open {
                return Err(AUTDInternalError::LinkClosed);
            }

            if self.recv_cnt > 10 {
                return Err(AUTDInternalError::LinkError("too many".to_owned()));
            }

            self.recv_cnt += 1;
            rx.iter_mut().for_each(|r| r.ack = self.recv_cnt as u8);

            Ok(!self.down)
        }

        #[cfg_attr(coverage_nightly, coverage(off))]
        fn is_open(&self) -> bool {
            self.is_open
        }

        fn timeout(&self) -> Duration {
            self.timeout
        }
    }

    #[tokio::test]
    async fn close() {
        let mut link = MockLink {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        assert!(link.is_open());

        link.close().await.unwrap();

        assert!(!link.is_open());
    }

    #[tokio::test]
    async fn test_send_receive() {
        let mut link = MockLink {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let tx = TxDatagram::new(0);
        let mut rx = Vec::new();
        assert_eq!(send_receive(&mut link, &tx, &mut rx, None).await, Ok(true));

        link.is_open = false;
        assert_eq!(
            send_receive(&mut link, &tx, &mut rx, None).await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.is_open = true;
        link.down = true;
        assert_eq!(send_receive(&mut link, &tx, &mut rx, None).await, Ok(false));

        link.down = false;
        assert_eq!(
            send_receive(&mut link, &tx, &mut rx, Some(Duration::from_millis(1))).await,
            Ok(true)
        );
    }

    #[tokio::test]
    async fn test_wait_msg_processed() {
        let mut link = MockLink {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let mut tx = TxDatagram::new(1);
        tx.header_mut(0).msg_id = 2;
        let mut rx = vec![RxMessage { ack: 0, data: 0 }];
        assert_eq!(
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::from_millis(10)).await,
            Ok(true)
        );

        link.recv_cnt = 0;
        link.is_open = false;
        assert_eq!(
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::from_millis(10)).await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::from_millis(10)).await,
            Ok(false)
        );

        link.down = false;
        link.recv_cnt = 0;
        tx.header_mut(0).msg_id = 20;
        assert_eq!(
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::from_secs(10)).await,
            Err(AUTDInternalError::LinkError("too many".to_owned()))
        );
    }
}
