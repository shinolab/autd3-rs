#![allow(clippy::too_many_arguments)]

mod instant;
mod sleep;

use std::time::Duration;

use autd3_driver::{
    derive::Geometry,
    error::AUTDInternalError,
    firmware::{
        cpu::{check_if_msg_is_processed, RxMessage, TxMessage},
        operation::{Operation, OperationHandler},
    },
    link::Link,
};
use instant::Instant;

use itertools::Itertools;
use sleep::Sleeper;
#[cfg(target_os = "windows")]
pub use sleep::WaitableSleeper;
pub use sleep::{AsyncSleeper, StdSleeper};
pub use spin_sleep::SpinSleeper;

use crate::error::AUTDError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimerStrategy {
    Std(StdSleeper),
    #[cfg(target_os = "windows")]
    Waitable(WaitableSleeper),
    Spin(SpinSleeper),
    Async(AsyncSleeper),
}

pub(crate) struct Timer {
    pub(crate) send_interval: Duration,
    pub(crate) receive_interval: Duration,
    pub(crate) strategy: TimerStrategy,
}

impl Timer {
    pub(crate) async fn send(
        &self,
        geometry: &Geometry,
        tx: &mut [TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        operations: Vec<(impl Operation, impl Operation)>,
        timeout: Duration,
        parallel: bool,
    ) -> Result<(), AUTDError> {
        match &self.strategy {
            TimerStrategy::Std(sleeper) => {
                self._send(
                    sleeper, geometry, tx, rx, link, operations, timeout, parallel,
                )
                .await
            }
            TimerStrategy::Spin(sleeper) => {
                self._send(
                    sleeper, geometry, tx, rx, link, operations, timeout, parallel,
                )
                .await
            }
            TimerStrategy::Async(sleeper) => {
                self._send(
                    sleeper, geometry, tx, rx, link, operations, timeout, parallel,
                )
                .await
            }
            #[cfg(target_os = "windows")]
            TimerStrategy::Waitable(sleeper) => {
                self._send(
                    sleeper, geometry, tx, rx, link, operations, timeout, parallel,
                )
                .await
            }
        }
    }

    async fn _send<S: Sleeper>(
        &self,
        sleeper: &S,
        geometry: &Geometry,
        tx: &mut [TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        mut operations: Vec<(impl Operation, impl Operation)>,
        timeout: Duration,
        parallel: bool,
    ) -> Result<(), AUTDError> {
        let mut send_timing = S::Instant::now();
        loop {
            OperationHandler::pack(&mut operations, geometry, tx, parallel)?;

            self.send_receive(sleeper, tx, rx, link, timeout).await?;

            if OperationHandler::is_done(&operations) {
                return Ok(());
            }

            S::Instant::add(&mut send_timing, self.send_interval);
            sleeper.sleep_until(send_timing).await;
        }
    }

    async fn send_receive(
        &self,
        sleeper: &impl Sleeper,
        tx: &[TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        timeout: Duration,
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
        self.wait_msg_processed(sleeper, tx, rx, link, timeout)
            .await
    }

    async fn wait_msg_processed<S: Sleeper>(
        &self,
        sleeper: &S,
        tx: &[TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        timeout: Duration,
    ) -> Result<(), AUTDInternalError> {
        let start = S::Instant::now();
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
            S::Instant::add(&mut receive_timing, self.receive_interval);
            sleeper.sleep_until(receive_timing).await;
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

    #[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
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

        fn is_open(&self) -> bool {
            self.is_open
        }
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

    #[rstest::rstest]
    #[case(TimerStrategy::Std(StdSleeper::default()), StdSleeper::default())]
    #[case(TimerStrategy::Spin(SpinSleeper::default()), SpinSleeper::default())]
    #[case(TimerStrategy::Async(AsyncSleeper::default()), AsyncSleeper::default())]
    #[cfg_attr(target_os = "windows", case(TimerStrategy::Waitable(WaitableSleeper::new().unwrap()), WaitableSleeper::new().unwrap()))]
    #[tokio::test]
    async fn test_send_receive(#[case] strategy: TimerStrategy, #[case] sleeper: impl Sleeper) {
        let mut link = MockLink {
            is_open: true,
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let tx = vec![];
        let mut rx = Vec::new();

        let timer = Timer {
            send_interval: Duration::from_millis(1),
            receive_interval: Duration::from_millis(1),
            strategy,
        };

        assert_eq!(
            timer
                .send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO)
                .await,
            Ok(())
        );

        link.is_open = false;
        assert_eq!(
            timer
                .send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO)
                .await,
            Err(AUTDInternalError::LinkClosed)
        );

        link.is_open = true;
        link.down = true;
        assert_eq!(
            timer
                .send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO)
                .await,
            Err(AUTDInternalError::SendDataFailed)
        );

        link.down = false;
        assert_eq!(
            timer
                .send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(1))
                .await,
            Ok(())
        );
    }

    #[rstest::rstest]
    #[case(TimerStrategy::Std(StdSleeper::default()), StdSleeper::default())]
    #[case(TimerStrategy::Spin(SpinSleeper::default()), SpinSleeper::default())]
    #[case(TimerStrategy::Async(AsyncSleeper::default()), AsyncSleeper::default())]
    #[cfg_attr(target_os = "windows", case(TimerStrategy::Waitable(WaitableSleeper::new().unwrap()), WaitableSleeper::new().unwrap()))]
    #[tokio::test]
    async fn test_wait_msg_processed(
        #[case] strategy: TimerStrategy,
        #[case] sleeper: impl Sleeper,
    ) {
        let mut link = MockLink {
            is_open: true,
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        let mut tx = vec![TxMessage::new_zeroed(); 1];
        tx[0].header_mut().msg_id = 2;
        let mut rx = vec![RxMessage::new(0, 0)];

        let timer = Timer {
            send_interval: Duration::from_millis(1),
            receive_interval: Duration::from_millis(1),
            strategy,
        };

        assert_eq!(
            timer
                .wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(10))
                .await,
            Ok(())
        );

        link.recv_cnt = 0;
        link.is_open = false;
        assert_eq!(
            Err(AUTDInternalError::LinkClosed),
            timer
                .wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(10))
                .await
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            Err(AUTDInternalError::ConfirmResponseFailed),
            timer
                .wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(10))
                .await,
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            Ok(()),
            timer
                .wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO)
                .await,
        );

        link.down = false;
        link.recv_cnt = 0;
        tx[0].header_mut().msg_id = 20;
        assert_eq!(
            Err(AUTDInternalError::LinkError("too many".to_owned())),
            timer
                .wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_secs(10))
                .await
        );
    }
}
