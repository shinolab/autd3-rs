use std::time::Duration;

use crate::{
    error::AUTDInternalError,
    firmware::cpu::{check_if_msg_is_processed, RxMessage, TxDatagram},
    geometry::Geometry,
};

use itertools::Itertools;

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    #[async_trait::async_trait]
    pub trait Link: Send {
        async fn close(&mut self) -> Result<(), AUTDInternalError>;

        async fn update(&mut self, _geometry: &Geometry) -> Result<(), AUTDInternalError> {
            Ok(())
        }

        async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError>;

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError>;

        #[must_use]
        fn is_open(&self) -> bool;

        #[must_use]
        fn timeout(&self) -> Duration;
        #[inline(always)]
        fn trace(&mut self, _: &TxDatagram, _: &mut [RxMessage], _: Duration, _: usize) {}
    }

    #[async_trait::async_trait]
    pub trait LinkBuilder: Send + Sync {
        type L: Link;

        async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError>;
    }

    #[async_trait::async_trait]
    impl Link for Box<dyn Link> {
        async fn close(&mut self) -> Result<(), AUTDInternalError> {
            self.as_mut().close().await
        }

        async fn update(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
            self.as_mut().update(geometry).await
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

        #[inline(always)]
        fn trace(
            &mut self,
            tx: &TxDatagram,
            rx: &mut [RxMessage],
            timeout: Duration,
            parallel_threshold: usize,
        ) {
            self.as_mut().trace(tx, rx, timeout, parallel_threshold)
        }
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    pub trait Link: Send {
        fn close(&mut self) -> impl std::future::Future<Output = Result<(), AUTDInternalError>>;

        fn update(
            &mut self,
            _geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<(), AUTDInternalError>> {
            async { Ok(()) }
        }

        fn send(
            &mut self,
            tx: &TxDatagram,
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;

        fn receive(
            &mut self,
            rx: &mut [RxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;

        #[must_use]
        fn is_open(&self) -> bool;

        #[must_use]
        fn timeout(&self) -> Duration;
        #[inline(always)]
        fn trace(&mut self, _: &TxDatagram, _: &mut [RxMessage], _: Duration, _: usize) {}
    }

    pub trait LinkBuilder {
        type L: Link;

        fn open(
            self,
            geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<Self::L, AUTDInternalError>>;
    }
}

pub use internal::Link;
pub use internal::LinkBuilder;

#[tracing::instrument(skip(link, tx, rx, timeout))]
pub async fn send_receive(
    link: &mut impl Link,
    tx: &TxDatagram,
    rx: &mut [RxMessage],
    timeout: Duration,
) -> Result<(), AUTDInternalError> {
    if !link.is_open() {
        return Err(AUTDInternalError::LinkClosed);
    }
    if !link.send(tx).await? {
        return Err(AUTDInternalError::SendDataFailed);
    }
    wait_msg_processed(link, tx, rx, timeout).await
}

async fn wait_msg_processed(
    link: &mut impl Link,
    tx: &TxDatagram,
    rx: &mut [RxMessage],
    timeout: Duration,
) -> Result<(), AUTDInternalError> {
    let start = std::time::Instant::now();
    loop {
        if !link.is_open() {
            return Err(AUTDInternalError::LinkClosed);
        }
        let res = link.receive(rx).await?;

        // GRCOV_EXCL_START
        tracing::trace!(
            "receive: {}",
            rx.iter()
                .format_with(", ", |elt, f| f(&format_args!("{:?}", elt)))
        );
        // GRCOV_EXCL_STOP

        if res && check_if_msg_is_processed(tx, rx).all(std::convert::identity) {
            return Ok(());
        }
        if start.elapsed() > timeout {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
    rx.iter()
        .try_fold((), |_, r| Result::<(), AUTDInternalError>::from(r))
        .and_then(|_| {
            if timeout == Duration::ZERO {
                Ok(())
            } else {
                Err(AUTDInternalError::ConfirmResponseFailed)
            }
        })
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

    #[cfg_attr(feature = "async-trait", async_trait::async_trait)]
    impl Link for MockLink {
        async fn close(&mut self) -> Result<(), AUTDInternalError> {
            self.is_open = false;
            Ok(())
        }

        async fn send(&mut self, _: &TxDatagram) -> Result<bool, AUTDInternalError> {
            self.send_cnt += 1;
            Ok(!self.down)
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
            if self.recv_cnt > 10 {
                return Err(AUTDInternalError::LinkError("too many".to_owned()));
            }

            self.recv_cnt += 1;
            rx.iter_mut()
                .for_each(|r| *r = RxMessage::new(self.recv_cnt as u8, r.data()));

            Ok(!self.down)
        }

        // GRCOV_EXCL_START
        fn is_open(&self) -> bool {
            self.is_open
        }

        fn timeout(&self) -> Duration {
            self.timeout
        }
        // GRCOV_EXCL_STOP
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_close() -> anyhow::Result<()> {
        let mut link = MockLink {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        assert!(link.is_open());

        link.close().await?;

        assert!(!link.is_open());

        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
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
        assert_eq!(
            send_receive(&mut link, &tx, &mut rx, Duration::ZERO).await,
            Ok(())
        );

        link.is_open = false;
        assert_eq!(
            send_receive(&mut link, &tx, &mut rx, Duration::ZERO).await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.is_open = true;
        link.down = true;
        assert_eq!(
            send_receive(&mut link, &tx, &mut rx, Duration::ZERO).await,
            Err(AUTDInternalError::SendDataFailed)
        );

        link.down = false;
        assert_eq!(
            send_receive(&mut link, &tx, &mut rx, Duration::from_millis(1)).await,
            Ok(())
        );
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_wait_msg_processed() {
        let mut link = MockLink {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let mut tx = TxDatagram::new(1);
        tx[0].header.msg_id = 2;
        let mut rx = vec![RxMessage::new(0, 0)];
        assert_eq!(
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::from_millis(10)).await,
            Ok(())
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
            Err(AUTDInternalError::ConfirmResponseFailed),
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::from_millis(10)).await,
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            Ok(()),
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::ZERO).await,
        );

        link.down = false;
        link.recv_cnt = 0;
        tx[0].header.msg_id = 20;
        assert_eq!(
            wait_msg_processed(&mut link, &tx, &mut rx, Duration::from_secs(10)).await,
            Err(AUTDInternalError::LinkError("too many".to_owned()))
        );
    }
}
