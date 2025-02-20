pub(crate) mod sleep;

use sleep::AsyncSleep;
pub use sleep::AsyncSleeper;

use std::time::{Duration, Instant};

use autd3_core::{datagram::Datagram, geometry::Geometry, link::AsyncLink};
use autd3_driver::{
    error::AUTDDriverError,
    firmware::{
        cpu::{RxMessage, TxMessage, check_if_msg_is_processed},
        operation::{Operation, OperationGenerator, OperationHandler},
    },
};

use itertools::Itertools;

use crate::controller::SenderOption;

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: AsyncLink, S: AsyncSleep> {
    pub(crate) link: &'a mut L,
    pub(crate) geometry: &'a mut Geometry,
    pub(crate) tx: &'a mut [TxMessage],
    pub(crate) rx: &'a mut [RxMessage],
    pub(crate) option: SenderOption<S>,
}

impl<L: AsyncLink, S: AsyncSleep> Sender<'_, L, S> {
    /// Send the [`Datagram`] to the devices.
    ///
    /// If the `timeout` value is
    /// - greater than 0, this function waits until the sent data is processed by the device or the specified timeout time elapses. If it cannot be confirmed that the sent data has been processed by the device, [`AUTDDriverError::ConfirmResponseFailed`] is returned.
    /// - 0, this function does not check whether the sent data has been processed by the device.
    ///
    /// The calculation of each [`Datagram`] is executed in parallel for each device if the number of enabled devices is greater than the `parallel_threshold`.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        let timeout = self.option.timeout.unwrap_or(s.option().timeout);
        let parallel = self
            .option
            .parallel
            .is_parallel(self.geometry.num_devices(), s.option().parallel_threshold);
        tracing::debug!("timeout: {:?}, parallel: {:?}", timeout, parallel);

        self.send_impl(
            OperationHandler::generate(
                s.operation_generator(self.geometry, parallel)?,
                self.geometry,
            ),
            timeout,
            parallel,
        )
        .await
    }

    pub(crate) async fn send_impl<O1, O2>(
        &mut self,
        mut operations: Vec<Option<(O1, O2)>>,
        timeout: Duration,
        parallel: bool,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        self.link.update(self.geometry).await?;

        // We prioritize average behavior for the transmission timing. That is, not the interval from the previous transmission, but ensuring that T/`send_interval` transmissions are performed in a sufficiently long time T.
        // For example, if the `send_interval` is 1ms and it takes 1.5ms to transmit due to some reason, the next transmission will be performed not 1ms later but 0.5ms later.
        let mut send_timing = Instant::now();
        loop {
            OperationHandler::pack(&mut operations, self.geometry, self.tx, parallel)?;

            self.send_receive(timeout).await?;

            if OperationHandler::is_done(&operations) {
                return Ok(());
            }

            send_timing += self.option.send_interval;
            self.option.sleeper.sleep_until(send_timing).await;
        }
    }

    async fn send_receive(&mut self, timeout: Duration) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Err(AUTDDriverError::LinkClosed);
        }

        tracing::trace!("send: {}", self.tx.iter().join(", "));
        self.link.send(self.tx).await?;
        self.wait_msg_processed(timeout).await
    }

    async fn wait_msg_processed(&mut self, timeout: Duration) -> Result<(), AUTDDriverError> {
        let start = Instant::now();
        let mut receive_timing = start;
        loop {
            if !self.link.is_open() {
                return Err(AUTDDriverError::LinkClosed);
            }
            self.link.receive(self.rx).await?;
            tracing::trace!("recv: {}", self.rx.iter().join(", "));

            if check_if_msg_is_processed(self.tx, self.rx).all(std::convert::identity) {
                return Ok(());
            }
            if start.elapsed() > timeout {
                break;
            }
            receive_timing += self.option.receive_interval;
            self.option.sleeper.sleep_until(receive_timing).await;
        }
        self.rx
            .iter()
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
    use spin_sleep::SpinSleeper;
    use zerocopy::FromZeros;

    #[cfg(target_os = "windows")]
    use crate::controller::WaitableSleeper;
    use crate::{
        controller::{ParallelMode, StdSleeper},
        tests::create_geometry,
    };

    use super::*;

    #[derive(Default)]
    struct MockAsyncLink {
        pub is_open: bool,
        pub send_cnt: usize,
        pub recv_cnt: usize,
        pub down: bool,
    }

    #[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
    impl AsyncLink for MockAsyncLink {
        async fn open(&mut self, _: &Geometry) -> Result<(), LinkError> {
            self.is_open = true;
            self.send_cnt = 0;
            self.recv_cnt = 0;
            self.down = false;
            Ok(())
        }

        async fn close(&mut self) -> Result<(), LinkError> {
            self.is_open = false;
            Ok(())
        }

        async fn send(&mut self, _: &[TxMessage]) -> Result<(), LinkError> {
            if !self.down {
                self.send_cnt += 1;
            }
            Ok(())
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
            if self.recv_cnt > 10 {
                return Err(LinkError::new("too many"));
            }
            if !self.down {
                self.recv_cnt += 1;
            }
            rx.iter_mut()
                .for_each(|r| *r = RxMessage::new(r.data(), self.recv_cnt as u8));
            Ok(())
        }

        fn is_open(&self) -> bool {
            self.is_open
        }
    }

    #[tokio::test]
    async fn test_close() -> anyhow::Result<()> {
        let mut link = MockAsyncLink::default();
        link.open(&Geometry::new(Vec::new())).await?;

        assert!(link.is_open());

        link.close().await?;

        assert!(!link.is_open());

        Ok(())
    }

    #[rstest::rstest]
    #[case(StdSleeper::default())]
    #[case(SpinSleeper::default())]
    #[case(AsyncSleeper::default())]
    #[cfg_attr(target_os = "windows", case(WaitableSleeper::new().unwrap()))]
    #[tokio::test]
    async fn test_send_receive(#[case] sleeper: impl AsyncSleep) {
        let mut link = MockAsyncLink::default();
        let mut geometry = create_geometry(1);
        let mut tx = vec![];
        let mut rx = Vec::new();

        assert!(link.open(&geometry).await.is_ok());
        let mut sender = Sender {
            link: &mut link,
            geometry: &mut geometry,
            tx: &mut tx,
            rx: &mut rx,
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel: ParallelMode::Auto,
                sleeper,
            },
        };

        assert_eq!(Ok(()), sender.send_receive(Duration::ZERO).await);
        assert_eq!(Ok(()), sender.send_receive(Duration::from_millis(1)).await);

        sender.link.is_open = false;
        assert_eq!(
            Err(AUTDDriverError::LinkClosed),
            sender.send_receive(Duration::ZERO).await,
        );
    }

    #[rstest::rstest]
    #[case(StdSleeper::default())]
    #[case(SpinSleeper::default())]
    #[case(AsyncSleeper::default())]
    #[cfg_attr(target_os = "windows", case(WaitableSleeper::new().unwrap()))]
    #[tokio::test]
    async fn test_wait_msg_processed(#[case] sleeper: impl AsyncSleep) {
        let mut link = MockAsyncLink::default();
        let mut geometry = create_geometry(1);
        let mut tx = vec![TxMessage::new_zeroed(); 1];
        tx[0].header.msg_id = 2;
        let mut rx = vec![RxMessage::new(0, 0)];

        assert!(link.open(&geometry).await.is_ok());
        let mut sender = Sender {
            link: &mut link,
            geometry: &mut geometry,
            tx: &mut tx,
            rx: &mut rx,
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel: ParallelMode::Auto,
                sleeper,
            },
        };

        assert_eq!(
            Ok(()),
            sender.wait_msg_processed(Duration::from_millis(10)).await,
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = false;
        assert_eq!(
            Err(AUTDDriverError::LinkClosed),
            sender.wait_msg_processed(Duration::from_millis(10)).await
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(
            Err(AUTDDriverError::ConfirmResponseFailed),
            sender.wait_msg_processed(Duration::from_millis(10)).await,
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(Ok(()), sender.wait_msg_processed(Duration::ZERO).await);

        sender.link.down = false;
        sender.link.recv_cnt = 0;
        sender.tx[0].header.msg_id = 20;
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("too many"))),
            sender.wait_msg_processed(Duration::from_secs(10)).await
        );
    }
}
