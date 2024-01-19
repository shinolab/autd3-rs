/*
 * File: link.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 18/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use std::time::Duration;

use crate::{
    cpu::{RxMessage, TxDatagram},
    error::AUTDInternalError,
    geometry::Geometry,
};

#[cfg(all(feature = "rpitit", feature = "async-trait"))]
compile_error!("`rpitit` and `async-trait` features are mutually exclusive");

#[cfg(not(any(feature = "rpitit", feature = "async-trait")))]
compile_error!("`rpitit` or `async-trait` feature is required");

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
        /// Send and receive data
        async fn send_receive(
            &mut self,
            tx: &TxDatagram,
            rx: &mut [RxMessage],
            timeout: Option<Duration>,
            ignore_ack: bool,
        ) -> Result<bool, AUTDInternalError> {
            let timeout = timeout.unwrap_or(self.timeout());
            if !self.send(tx).await? {
                return Ok(false);
            }
            if timeout.is_zero() {
                return self.receive(rx).await;
            }
            self.wait_msg_processed(tx, rx, timeout, ignore_ack).await
        }

        /// Wait until message is processed
        async fn wait_msg_processed(
            &mut self,
            tx: &TxDatagram,
            rx: &mut [RxMessage],
            timeout: Duration,
            ignore_ack: bool,
        ) -> Result<bool, AUTDInternalError> {
            let start = std::time::Instant::now();
            let _ = self.receive(rx).await?;
            if tx.headers().zip(rx.iter()).try_fold(true, |acc, (h, r)| {
                if !ignore_ack && r.ack & 0x80 != 0 {
                    return Err(AUTDInternalError::firmware_err(r.ack));
                }
                Ok(acc && h.msg_id == r.ack)
            })? {
                return Ok(true);
            }
            loop {
                if start.elapsed() > timeout {
                    return Ok(false);
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                if !self.receive(rx).await? {
                    continue;
                }
                if tx.headers().zip(rx.iter()).try_fold(true, |acc, (h, r)| {
                    if !ignore_ack && r.ack & 0x80 != 0 {
                        return Err(AUTDInternalError::firmware_err(r.ack));
                    }
                    Ok(acc && h.msg_id == r.ack)
                })? {
                    return Ok(true);
                }
            }
        }
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

#[cfg(feature = "rpitit")]
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
        /// Send and receive data
        fn send_receive(
            &mut self,
            tx: &TxDatagram,
            rx: &mut [RxMessage],
            timeout: Option<Duration>,
            ignore_ack: bool,
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>> {
            async move {
                let timeout = timeout.unwrap_or(self.timeout());
                if !self.send(tx).await? {
                    return Ok(false);
                }
                if timeout.is_zero() {
                    return self.receive(rx).await;
                }
                self.wait_msg_processed(tx, rx, timeout, ignore_ack).await
            }
        }

        /// Wait until message is processed
        fn wait_msg_processed(
            &mut self,
            tx: &TxDatagram,
            rx: &mut [RxMessage],
            timeout: Duration,
            ignore_ack: bool,
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>> {
            async move {
                let start = std::time::Instant::now();
                let _ = self.receive(rx).await?;
                if tx.headers().zip(rx.iter()).try_fold(true, |acc, (h, r)| {
                    if !ignore_ack && r.ack & 0x80 != 0 {
                        return Err(AUTDInternalError::firmware_err(r.ack));
                    }
                    Ok(acc && h.msg_id == r.ack)
                })? {
                    return Ok(true);
                }
                loop {
                    if start.elapsed() > timeout {
                        return Ok(false);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    if !self.receive(rx).await? {
                        continue;
                    }
                    if tx.headers().zip(rx.iter()).try_fold(true, |acc, (h, r)| {
                        if !ignore_ack && r.ack & 0x80 != 0 {
                            return Err(AUTDInternalError::firmware_err(r.ack));
                        }
                        Ok(acc && h.msg_id == r.ack)
                    })? {
                        return Ok(true);
                    }
                }
            }
        }
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

#[cfg(feature = "rpitit")]
pub use internal::Link;
#[cfg(feature = "rpitit")]
pub use internal::LinkBuilder;

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
    async fn send_receive() {
        let mut link = MockLink {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let tx = TxDatagram::new(0);
        let mut rx = Vec::new();
        assert_eq!(link.send_receive(&tx, &mut rx, None, false).await, Ok(true));

        link.is_open = false;
        assert_eq!(
            link.send_receive(&tx, &mut rx, None, false).await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.is_open = true;
        link.down = true;
        assert_eq!(
            link.send_receive(&tx, &mut rx, None, false).await,
            Ok(false)
        );

        link.down = false;
        assert_eq!(
            link.send_receive(&tx, &mut rx, Some(Duration::from_millis(1)), false)
                .await,
            Ok(true)
        );
    }

    #[tokio::test]
    async fn wait_msg_processed() {
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
            link.wait_msg_processed(&tx, &mut rx, Duration::from_millis(10), false)
                .await,
            Ok(true)
        );

        link.recv_cnt = 0;
        link.is_open = false;
        assert_eq!(
            link.wait_msg_processed(&tx, &mut rx, Duration::from_millis(10), false)
                .await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            link.wait_msg_processed(&tx, &mut rx, Duration::from_millis(10), false)
                .await,
            Ok(false)
        );

        link.down = false;
        link.recv_cnt = 0;
        tx.header_mut(0).msg_id = 20;
        assert_eq!(
            link.wait_msg_processed(&tx, &mut rx, Duration::from_secs(10), false)
                .await,
            Err(AUTDInternalError::LinkError("too many".to_owned()))
        );
    }
}
