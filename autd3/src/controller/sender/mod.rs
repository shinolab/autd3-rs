pub(crate) mod sleep;

use sleep::Sleep;
#[cfg(target_os = "windows")]
pub use sleep::WaitableSleeper;
pub use sleep::{SpinSleeper, StdSleeper};
pub use spin_sleep::SpinStrategy;

use std::time::{Duration, Instant};

use autd3_core::{datagram::Datagram, derive::DatagramOption, geometry::Geometry, link::Link};
use autd3_driver::{
    error::AUTDDriverError,
    firmware::{
        cpu::{check_if_msg_is_processed, RxMessage, TxMessage},
        operation::{Operation, OperationGenerator, OperationHandler},
    },
};

use itertools::Itertools;

/// The option of [`Sender`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SenderOption<S> {
    /// The duration between sending operations.
    pub send_interval: Duration,
    /// The duration between receiving operations.
    pub receive_interval: Duration,
    /// If `None`, [`Datagram::option`] is used.
    ///
    /// [`Datagram`]: autd3_driver::datagram::Datagram
    pub timeout: Option<Duration>,
    /// If `None`, [`Datagram::option`] is used.
    ///
    /// [`Datagram`]: autd3_driver::datagram::Datagram
    pub parallel_threshold: Option<usize>,
    /// The sleeper to manage the sending/receiving timing.
    pub sleeper: S,
}

impl<S: Default> Default for SenderOption<S> {
    fn default() -> Self {
        Self {
            send_interval: Duration::from_millis(1),
            receive_interval: Duration::from_millis(1),
            timeout: None,
            parallel_threshold: None,
            sleeper: S::default(),
        }
    }
}

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: Link, S: Sleep> {
    pub(crate) link: &'a mut L,
    pub(crate) geometry: &'a mut Geometry,
    pub(crate) tx: &'a mut [TxMessage],
    pub(crate) rx: &'a mut [RxMessage],
    pub(crate) option: SenderOption<S>,
}

impl<L: Link, S: Sleep> Sender<'_, L, S> {
    /// Send the [`Datagram`] to the devices.
    ///
    /// If the `timeout` value is
    /// - greater than 0, this function waits until the sent data is processed by the device or the specified timeout time elapses. If it cannot be confirmed that the sent data has been processed by the device, [`AUTDDriverError::ConfirmResponseFailed`] is returned.
    /// - 0, this function does not check whether the sent data has been processed by the device.
    ///
    /// The calculation of each [`Datagram`] is executed in parallel for each device if the number of enabled devices is greater than the `parallel_threshold`.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        let timeout = self.option.timeout.unwrap_or(s.option().timeout);
        let parallel_threshold = self
            .option
            .parallel_threshold
            .unwrap_or(s.option().parallel_threshold);
        let datagram_option = DatagramOption {
            timeout,
            parallel_threshold,
        };
        self.send_impl(
            OperationHandler::generate(
                s.operation_generator(self.geometry, &datagram_option)?,
                self.geometry,
            ),
            &datagram_option,
        )
    }

    pub(crate) fn send_impl<O1, O2>(
        &mut self,
        mut operations: Vec<(O1, O2)>,
        option: &DatagramOption,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        let timeout = option.timeout;
        let parallel_threshold = option.parallel_threshold;

        let parallel = self.geometry.num_devices() > parallel_threshold;

        self.link.trace(option);
        tracing::debug!("timeout: {:?}, parallel: {:?}", timeout, parallel);

        self.link.update(self.geometry)?;

        // We prioritize average behavior for the transmission timing. That is, not the interval from the previous transmission, but ensuring that T/`send_interval` transmissions are performed in a sufficiently long time T.
        // For example, if the `send_interval` is 1ms and it takes 1.5ms to transmit due to some reason, the next transmission will be performed not 1ms later but 0.5ms later.
        let mut send_timing = Instant::now();
        loop {
            OperationHandler::pack(&mut operations, self.geometry, self.tx, parallel)?;

            self.send_receive(timeout)?;

            if OperationHandler::is_done(&operations) {
                return Ok(());
            }

            send_timing += self.option.send_interval;
            self.option.sleeper.sleep_until(send_timing);
        }
    }

    fn send_receive(&mut self, timeout: Duration) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Err(AUTDDriverError::LinkClosed);
        }

        tracing::trace!("send: {}", self.tx.iter().join(", "));
        if !self.link.send(self.tx)? {
            return Err(AUTDDriverError::SendDataFailed);
        }
        self.wait_msg_processed(timeout)
    }

    fn wait_msg_processed(&mut self, timeout: Duration) -> Result<(), AUTDDriverError> {
        let start = Instant::now();
        let mut receive_timing = start;
        loop {
            if !self.link.is_open() {
                return Err(AUTDDriverError::LinkClosed);
            }
            let res = self.link.receive(self.rx)?;
            tracing::trace!("recv: {}", self.rx.iter().join(", "));

            if res && check_if_msg_is_processed(self.tx, self.rx).all(std::convert::identity) {
                return Ok(());
            }
            if start.elapsed() > timeout {
                break;
            }
            receive_timing += self.option.receive_interval;
            self.option.sleeper.sleep_until(receive_timing);
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
    use zerocopy::FromZeros;

    #[cfg(target_os = "windows")]
    use crate::controller::sender::WaitableSleeper;
    use crate::{
        controller::sender::{SpinSleeper, StdSleeper},
        tests::create_geometry,
    };

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
    #[case(StdSleeper::default())]
    #[case(SpinSleeper::default())]
    #[cfg_attr(target_os = "windows", case(WaitableSleeper::new().unwrap()))]
    #[test]
    fn test_send_receive(#[case] sleeper: impl Sleep) {
        let mut link = MockLink {
            is_open: true,
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };
        let mut geometry = create_geometry(1);
        let mut tx = vec![];
        let mut rx = Vec::new();

        let mut sender = Sender {
            link: &mut link,
            geometry: &mut geometry,
            tx: &mut tx,
            rx: &mut rx,
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel_threshold: None,
                sleeper,
            },
        };

        assert_eq!(sender.send_receive(Duration::ZERO), Ok(()));

        sender.link.is_open = false;
        assert_eq!(
            sender.send_receive(Duration::ZERO),
            Err(AUTDDriverError::LinkClosed)
        );

        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(
            sender.send_receive(Duration::ZERO),
            Err(AUTDDriverError::SendDataFailed)
        );

        sender.link.down = false;
        assert_eq!(sender.send_receive(Duration::from_millis(1)), Ok(()));
    }

    #[rstest::rstest]
    #[case(StdSleeper::default())]
    #[case(SpinSleeper::default())]
    #[cfg_attr(target_os = "windows", case(WaitableSleeper::new().unwrap()))]
    #[test]
    fn test_wait_msg_processed(#[case] sleeper: impl Sleep) {
        let mut link = MockLink {
            is_open: true,
            send_cnt: 0,
            recv_cnt: 0,
            down: false,
        };
        let mut geometry = create_geometry(1);
        let mut tx = vec![TxMessage::new_zeroed(); 1];
        tx[0].header.msg_id = 2;
        let mut rx = vec![RxMessage::new(0, 0)];

        let mut sender = Sender {
            link: &mut link,
            geometry: &mut geometry,
            tx: &mut tx,
            rx: &mut rx,
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel_threshold: None,
                sleeper,
            },
        };

        assert_eq!(sender.wait_msg_processed(Duration::from_millis(10)), Ok(()));

        sender.link.recv_cnt = 0;
        sender.link.is_open = false;
        assert_eq!(
            Err(AUTDDriverError::LinkClosed),
            sender.wait_msg_processed(Duration::from_millis(10))
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(
            Err(AUTDDriverError::ConfirmResponseFailed),
            sender.wait_msg_processed(Duration::from_millis(10)),
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(Ok(()), sender.wait_msg_processed(Duration::ZERO),);

        sender.link.down = false;
        sender.link.recv_cnt = 0;
        sender.tx[0].header.msg_id = 20;
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("too many".to_owned()))),
            sender.wait_msg_processed(Duration::from_secs(10))
        );
    }
}
