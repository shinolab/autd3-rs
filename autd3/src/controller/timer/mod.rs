#![allow(clippy::too_many_arguments)]

pub(crate) mod sleep;

use autd3_core::{geometry::Geometry, link::Link};
use autd3_derive::Builder;
use sleep::Sleeper;
#[cfg(target_os = "windows")]
pub use sleep::WaitableSleeper;
pub use sleep::{SpinSleeper, StdSleeper};

use std::time::{Duration, Instant};

use autd3_driver::{
    error::AUTDDriverError,
    firmware::{
        cpu::{check_if_msg_is_processed, RxMessage, TxMessage},
        operation::{Operation, OperationHandler},
    },
};

use itertools::Itertools;

/// Enum representing sleeping strategies for the timer.
///
/// The [`TimerStrategy`] enum provides various strategies for implementing a timer
/// with different sleeping mechanisms. This allows for flexibility in how the timer
/// behaves depending on the target operating system and specific requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimerStrategy {
    /// Uses [`std::thread::sleep`].
    Std(StdSleeper),
    /// Uses a [waitable timer](https://learn.microsoft.com/en-us/windows/win32/sync/waitable-timer-objects) available only on Windows.
    #[cfg(target_os = "windows")]
    Waitable(WaitableSleeper),
    /// Uses a [spin_sleep](https://crates.io/crates/spin_sleep) crate.
    Spin(SpinSleeper),
}

/// A struct managing the timing of sending and receiving operations.
// Timer can be generic, but in that case, `Controller` must also be generic. To avoid this, `TimerStrategy` is an enum.
#[derive(Builder)]
pub struct Timer {
    #[get]
    /// The duration between sending operations.
    pub(crate) send_interval: Duration,
    #[get]
    /// The duration between receiving operations.
    pub(crate) receive_interval: Duration,
    #[get]
    /// The strategy used for timing operations.
    pub(crate) strategy: TimerStrategy,
    #[get]
    /// The default timeout when no timeout is specified for the [`Datagram`] to be sent.
    ///
    /// [`Datagram`]: autd3_driver::datagram::Datagram
    pub(crate) default_timeout: Duration,
}

impl Timer {
    pub(crate) fn send<O1, O2>(
        &self,
        geometry: &Geometry,
        tx: &mut [TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        operations: Vec<(O1, O2)>,
        timeout: Option<Duration>,
        parallel_threshold: Option<usize>,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        let timeout = timeout.unwrap_or(self.default_timeout);
        let parallel = geometry.parallel(parallel_threshold);
        tracing::debug!("timeout: {:?}, parallel: {:?}", timeout, parallel);

        match &self.strategy {
            TimerStrategy::Std(sleeper) => self._send(
                sleeper, geometry, tx, rx, link, operations, timeout, parallel,
            ),
            TimerStrategy::Spin(sleeper) => self._send(
                sleeper, geometry, tx, rx, link, operations, timeout, parallel,
            ),
            #[cfg(target_os = "windows")]
            TimerStrategy::Waitable(sleeper) => self._send(
                sleeper, geometry, tx, rx, link, operations, timeout, parallel,
            ),
        }
    }

    fn _send<O1, O2, S: Sleeper>(
        &self,
        sleeper: &S,
        geometry: &Geometry,
        tx: &mut [TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        mut operations: Vec<(O1, O2)>,
        timeout: Duration,
        parallel: bool,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        link.update(geometry)?;

        // We prioritize average behavior for the transmission timing. That is, not the interval from the previous transmission, but ensuring that T/`send_interval` transmissions are performed in a sufficiently long time T.
        // For example, if the `send_interval` is 1ms and it takes 1.5ms to transmit due to some reason, the next transmission will be performed not 1ms later but 0.5ms later.
        let mut send_timing = Instant::now();
        loop {
            OperationHandler::pack(&mut operations, geometry, tx, parallel)?;

            self.send_receive(sleeper, tx, rx, link, timeout)?;

            if OperationHandler::is_done(&operations) {
                return Ok(());
            }

            send_timing += self.send_interval;
            sleeper.sleep_until(send_timing);
        }
    }

    fn send_receive(
        &self,
        sleeper: &impl Sleeper,
        tx: &[TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        timeout: Duration,
    ) -> Result<(), AUTDDriverError> {
        if !link.is_open() {
            return Err(AUTDDriverError::LinkClosed);
        }

        tracing::trace!("send: {}", tx.iter().join(", "));
        if !link.send(tx)? {
            return Err(AUTDDriverError::SendDataFailed);
        }
        self.wait_msg_processed(sleeper, tx, rx, link, timeout)
    }

    fn wait_msg_processed<S: Sleeper>(
        &self,
        sleeper: &S,
        tx: &[TxMessage],
        rx: &mut [RxMessage],
        link: &mut impl Link,
        timeout: Duration,
    ) -> Result<(), AUTDDriverError> {
        let start = Instant::now();
        let mut receive_timing = start;
        loop {
            if !link.is_open() {
                return Err(AUTDDriverError::LinkClosed);
            }
            let res = link.receive(rx)?;
            tracing::trace!("recv: {}", rx.iter().join(", "));

            if res && check_if_msg_is_processed(tx, rx).all(std::convert::identity) {
                return Ok(());
            }
            if start.elapsed() > timeout {
                break;
            }
            receive_timing += self.receive_interval;
            sleeper.sleep_until(receive_timing);
        }
        rx.iter()
            .try_fold((), |_, r| {
                autd3_driver::firmware::cpu::check_firmware_err(r)
            })
            .and_then(|e| {
                if timeout == Duration::ZERO {
                    Ok(())
                } else {
                    tracing::error!("Failed to confirm the response from the device: {:?}", e);
                    Err(AUTDDriverError::ConfirmResponseFailed)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::link::LinkError;
    use zerocopy::FromZeros;

    #[cfg(target_os = "windows")]
    use crate::controller::timer::WaitableSleeper;
    use crate::controller::timer::{SpinSleeper, StdSleeper};

    use super::*;

    struct MockLink {
        pub is_open: bool,
        pub send_cnt: usize,
        pub recv_cnt: usize,
        pub down: bool,
    }

    impl Link for MockLink {
        fn close(&mut self) -> Result<(), LinkError> {
            self.is_open = false;
            Ok(())
        }

        fn send(&mut self, _: &[TxMessage]) -> Result<bool, LinkError> {
            self.send_cnt += 1;
            Ok(!self.down)
        }

        fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
            if self.recv_cnt > 10 {
                return Err(LinkError::new("too many".to_owned()));
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

    #[test]
    fn test_close() -> anyhow::Result<()> {
        let mut link = MockLink {
            is_open: true,
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };

        assert!(link.is_open());

        link.close()?;

        assert!(!link.is_open());

        Ok(())
    }

    #[rstest::rstest]
    #[case(TimerStrategy::Std(StdSleeper::default()), StdSleeper::default())]
    #[case(TimerStrategy::Spin(SpinSleeper::default()), SpinSleeper::default())]
    #[cfg_attr(target_os = "windows", case(TimerStrategy::Waitable(WaitableSleeper::new().unwrap()), WaitableSleeper::new().unwrap()))]
    #[test]
    fn test_send_receive(#[case] strategy: TimerStrategy, #[case] sleeper: impl Sleeper) {
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
            default_timeout: Duration::ZERO,
        };

        assert_eq!(
            timer.send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO),
            Ok(())
        );

        link.is_open = false;
        assert_eq!(
            timer.send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO),
            Err(AUTDDriverError::LinkClosed)
        );

        link.is_open = true;
        link.down = true;
        assert_eq!(
            timer.send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO),
            Err(AUTDDriverError::SendDataFailed)
        );

        link.down = false;
        assert_eq!(
            timer.send_receive(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(1)),
            Ok(())
        );
    }

    #[rstest::rstest]
    #[case(TimerStrategy::Std(StdSleeper::default()), StdSleeper::default())]
    #[case(TimerStrategy::Spin(SpinSleeper::default()), SpinSleeper::default())]
    #[cfg_attr(target_os = "windows", case(TimerStrategy::Waitable(WaitableSleeper::new().unwrap()), WaitableSleeper::new().unwrap()))]
    #[test]
    fn test_wait_msg_processed(#[case] strategy: TimerStrategy, #[case] sleeper: impl Sleeper) {
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
            default_timeout: Duration::ZERO,
        };

        assert_eq!(
            timer.wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(10)),
            Ok(())
        );

        link.recv_cnt = 0;
        link.is_open = false;
        assert_eq!(
            Err(AUTDDriverError::LinkClosed),
            timer.wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(10))
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            Err(AUTDDriverError::ConfirmResponseFailed),
            timer.wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_millis(10)),
        );

        link.recv_cnt = 0;
        link.is_open = true;
        link.down = true;
        assert_eq!(
            Ok(()),
            timer.wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::ZERO),
        );

        link.down = false;
        link.recv_cnt = 0;
        tx[0].header_mut().msg_id = 20;
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("too many".to_owned()))),
            timer.wait_msg_processed(&sleeper, &tx, &mut rx, &mut link, Duration::from_secs(10))
        );
    }
}
