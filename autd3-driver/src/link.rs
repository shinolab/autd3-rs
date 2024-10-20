use std::time::Duration;

use crate::{
    error::AUTDInternalError,
    firmware::cpu::{check_if_msg_is_processed, RxMessage, TxMessage},
    geometry::Geometry,
};

use itertools::Itertools;

#[cfg(feature = "async-trait")]
mod internal {
    use crate::firmware::cpu::TxMessage;

    use super::*;

    #[async_trait::async_trait]
    pub trait Link: Send {
        async fn close(&mut self) -> Result<(), AUTDInternalError>;

        async fn update(&mut self, _geometry: &Geometry) -> Result<(), AUTDInternalError> {
            Ok(())
        }

        async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDInternalError>;

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError>;

        #[must_use]
        fn is_open(&self) -> bool;

        #[inline(always)]
        fn trace(&mut self, _: &[TxMessage], _: &mut [RxMessage], _: Duration, _: usize) {}
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

        async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDInternalError> {
            self.as_mut().send(tx).await
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
            self.as_mut().receive(rx).await
        }

        fn is_open(&self) -> bool {
            self.as_ref().is_open()
        }

        #[inline(always)]
        fn trace(
            &mut self,
            tx: &[TxMessage],
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
    use crate::firmware::cpu::TxMessage;

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
            tx: &[TxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;

        fn receive(
            &mut self,
            rx: &mut [RxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;

        #[must_use]
        fn is_open(&self) -> bool;

        #[inline(always)]
        fn trace(&mut self, _: &[TxMessage], _: &mut [RxMessage], _: Duration, _: usize) {}
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

#[tracing::instrument(skip(link, tx, rx))]
pub async fn send_receive(
    link: &mut impl Link,
    tx: &[TxMessage],
    rx: &mut [RxMessage],
    timeout: Duration,
    receive_interval: Duration,
) -> Result<(), AUTDInternalError> {
    if !link.is_open() {
        return Err(AUTDInternalError::LinkClosed);
    }

    // GRCOV_EXCL_START
    tracing::trace!(
        "send: {}",
        tx.iter().format_with(", ", |elt, f| {
            f(&format_args!(
                "({:?}, TAG: {:#04X})",
                elt.header(),
                elt.payload()[0]
            ))
        })
    );
    // GRCOV_EXCL_STOP

    if !link.send(tx).await? {
        return Err(AUTDInternalError::SendDataFailed);
    }
    wait_msg_processed(link, tx, rx, timeout, receive_interval).await
}

async fn wait_msg_processed(
    link: &mut impl Link,
    tx: &[TxMessage],
    rx: &mut [RxMessage],
    timeout: Duration,
    receive_interval: Duration,
) -> Result<(), AUTDInternalError> {
    let start = tokio::time::Instant::now();
    let mut receive_timing = start;
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
        receive_timing += receive_interval;
        tokio::time::sleep_until(receive_timing).await;
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
    use zerocopy::FromZeros;

    use super::*;

    struct MockLink {
        pub is_open: bool,
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

        async fn send(&mut self, _: &[TxMessage]) -> Result<bool, AUTDInternalError> {
            self.send_cnt += 1;
            Ok(!self.down)
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
            if self.recv_cnt > 10 {
                return Err(AUTDInternalError::LinkError("too many".to_owned()));
            }

            self.recv_cnt += 1;
            rx.iter_mut()
                .for_each(|r| *r = RxMessage::new(r.data(), self.recv_cnt as u8));

            Ok(!self.down)
        }

        // GRCOV_EXCL_START
        fn is_open(&self) -> bool {
            self.is_open
        }
        // GRCOV_EXCL_STOP
    }

    #[tokio::test]
    async fn test_close() -> anyhow::Result<()> {
        let mut link = MockLink {
            is_open: true,
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
    async fn test_send_receive() {
        let mut link = MockLink {
            is_open: true,
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let tx = vec![];
        let mut rx = Vec::new();
        assert_eq!(
            send_receive(
                &mut link,
                &tx,
                &mut rx,
                Duration::ZERO,
                Duration::from_millis(1)
            )
            .await,
            Ok(())
        );

        link.is_open = false;
        assert_eq!(
            send_receive(
                &mut link,
                &tx,
                &mut rx,
                Duration::ZERO,
                Duration::from_millis(1)
            )
            .await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.is_open = true;
        link.down = true;
        assert_eq!(
            send_receive(
                &mut link,
                &tx,
                &mut rx,
                Duration::ZERO,
                Duration::from_millis(1)
            )
            .await,
            Err(AUTDInternalError::SendDataFailed)
        );

        link.down = false;
        assert_eq!(
            send_receive(
                &mut link,
                &tx,
                &mut rx,
                Duration::from_millis(1),
                Duration::from_millis(1)
            )
            .await,
            Ok(())
        );
    }

    #[tokio::test]
    async fn test_wait_msg_processed() {
        let mut link = MockLink {
            is_open: true,
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let mut tx = vec![TxMessage::new_zeroed(); 1];
        tx[0].header_mut().msg_id = 2;
        let mut rx = vec![RxMessage::new(0, 0)];
        assert_eq!(
            wait_msg_processed(
                &mut link,
                &tx,
                &mut rx,
                Duration::from_millis(10),
                Duration::from_millis(1)
            )
            .await,
            Ok(())
        );

        link.recv_cnt = 0;
        link.is_open = false;
        assert_eq!(
            wait_msg_processed(
                &mut link,
                &tx,
                &mut rx,
                Duration::from_millis(10),
                Duration::from_millis(1)
            )
            .await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            Err(AUTDInternalError::ConfirmResponseFailed),
            wait_msg_processed(
                &mut link,
                &tx,
                &mut rx,
                Duration::from_millis(10),
                Duration::from_millis(1)
            )
            .await,
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            Ok(()),
            wait_msg_processed(
                &mut link,
                &tx,
                &mut rx,
                Duration::ZERO,
                Duration::from_millis(1)
            )
            .await,
        );

        link.down = false;
        link.recv_cnt = 0;
        tx[0].header_mut().msg_id = 20;
        assert_eq!(
            wait_msg_processed(
                &mut link,
                &tx,
                &mut rx,
                Duration::from_secs(10),
                Duration::from_millis(1)
            )
            .await,
            Err(AUTDInternalError::LinkError("too many".to_owned()))
        );
    }
}
