/*
 * File: link.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use std::time::Duration;

use crate::{
    cpu::{RxMessage, TxDatagram},
    derive::prelude::Geometry,
    error::AUTDInternalError,
};

/// Link is a interface to the AUTD device
pub trait Link: Send + Sync {
    /// Close link
    fn close(&mut self) -> impl std::future::Future<Output = Result<(), AUTDInternalError>> + Send;
    /// Send data to devices
    fn send(
        &mut self,
        tx: &TxDatagram,
    ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>> + Send;
    /// Receive data from devices
    fn receive(
        &mut self,
        rx: &mut [RxMessage],
    ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>> + Send;
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
    ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>> + Send {
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
    ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>> + Send {
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
                std::thread::sleep(std::time::Duration::from_millis(1));
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

#[cfg(feature = "sync")]
/// Link for blocking operation
pub trait LinkSync {
    fn close(&mut self) -> Result<(), AUTDInternalError>;
    fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError>;
    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError>;
    #[must_use]
    fn is_open(&self) -> bool;
    #[must_use]
    fn timeout(&self) -> Duration;
    fn send_receive(
        &mut self,
        tx: &TxDatagram,
        rx: &mut [RxMessage],
        timeout: Option<Duration>,
        ignore_ack: bool,
    ) -> Result<bool, AUTDInternalError> {
        let timeout = timeout.unwrap_or(self.timeout());
        if !self.send(tx)? {
            return Ok(false);
        }
        if timeout.is_zero() {
            return self.receive(rx);
        }
        self.wait_msg_processed(tx, rx, timeout, ignore_ack)
    }
    fn wait_msg_processed(
        &mut self,
        tx: &TxDatagram,
        rx: &mut [RxMessage],
        timeout: Duration,
        ignore_ack: bool,
    ) -> Result<bool, AUTDInternalError> {
        let start = std::time::Instant::now();
        let _ = self.receive(rx)?;

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
            std::thread::sleep(std::time::Duration::from_millis(1));
            if !self.receive(rx)? {
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

pub trait LinkBuilder {
    type L: Link;

    /// Open link
    fn open(
        self,
        geometry: &Geometry,
    ) -> impl std::future::Future<Output = Result<Self::L, AUTDInternalError>> + Send;
}

#[cfg(feature = "sync")]
pub trait LinkSyncBuilder {
    type L: LinkSync;

    /// Open link
    fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError>;
}

#[cfg(feature = "sync")]
impl LinkSync for Box<dyn LinkSync> {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.as_mut().close()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        self.as_mut().send(tx)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        self.as_mut().receive(rx)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn is_open(&self) -> bool {
        self.as_ref().is_open()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn timeout(&self) -> Duration {
        self.as_ref().timeout()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn send_receive(
        &mut self,
        tx: &TxDatagram,
        rx: &mut [RxMessage],
        timeout: Option<Duration>,
        ignore_ack: bool,
    ) -> Result<bool, AUTDInternalError> {
        self.as_mut().send_receive(tx, rx, timeout, ignore_ack)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn wait_msg_processed(
        &mut self,
        tx: &TxDatagram,
        rx: &mut [RxMessage],
        timeout: Duration,
        ignore_ack: bool,
    ) -> Result<bool, AUTDInternalError> {
        self.as_mut()
            .wait_msg_processed(tx, rx, timeout, ignore_ack)
    }
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

    #[cfg(feature = "sync")]
    struct MockLinkSync {
        pub is_open: bool,
        pub timeout: Duration,
        pub send_cnt: usize,
        pub recv_cnt: usize,
        pub down: bool,
    }

    #[cfg(feature = "sync")]
    impl LinkSync for MockLinkSync {
        fn close(&mut self) -> Result<(), AUTDInternalError> {
            self.is_open = false;
            Ok(())
        }

        fn send(&mut self, _: &TxDatagram) -> Result<bool, AUTDInternalError> {
            if !self.is_open {
                return Err(AUTDInternalError::LinkClosed);
            }

            self.send_cnt += 1;
            Ok(!self.down)
        }

        fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
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

    #[test]
    #[cfg(feature = "sync")]
    fn close_sync() {
        let mut link = MockLinkSync {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        assert!(link.is_open());

        link.close().unwrap();

        assert!(!link.is_open());
    }

    #[test]
    #[cfg(feature = "sync")]
    fn send_receive_sync() {
        let mut link = MockLinkSync {
            is_open: true,
            timeout: Duration::from_millis(0),
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let tx = TxDatagram::new(0);
        let mut rx = Vec::new();
        assert_eq!(link.send_receive(&tx, &mut rx, None, false), Ok(true));

        link.is_open = false;
        assert_eq!(
            link.send_receive(&tx, &mut rx, None, false),
            Err(AUTDInternalError::LinkClosed)
        );

        link.is_open = true;
        link.down = true;
        assert_eq!(link.send_receive(&tx, &mut rx, None, false), Ok(false));

        link.down = false;
        assert_eq!(
            link.send_receive(&tx, &mut rx, Some(Duration::from_millis(1)), false),
            Ok(true)
        );
    }

    #[test]
    #[cfg(feature = "sync")]
    fn wait_msg_processed_sync() {
        let mut link = MockLinkSync {
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
            link.wait_msg_processed(&tx, &mut rx, Duration::from_millis(10), false),
            Ok(true)
        );

        link.recv_cnt = 0;
        link.is_open = false;
        assert_eq!(
            link.wait_msg_processed(&tx, &mut rx, Duration::from_millis(10), false),
            Err(AUTDInternalError::LinkClosed)
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            link.wait_msg_processed(&tx, &mut rx, Duration::from_millis(10), false),
            Ok(false)
        );

        link.down = false;
        link.recv_cnt = 0;
        tx.header_mut(0).msg_id = 20;
        assert_eq!(
            link.wait_msg_processed(&tx, &mut rx, Duration::from_secs(10), false),
            Err(AUTDInternalError::LinkError("too many".to_owned()))
        );
    }
}
