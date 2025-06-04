pub(crate) mod sleep;

use sleep::Sleep;
pub use sleep::{SpinSleeper, StdSleeper};
pub use spin_sleep::SpinStrategy;

use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use autd3_core::{
    datagram::Datagram,
    derive::DeviceFilter,
    geometry::Geometry,
    link::{Link, MsgId},
};
use autd3_driver::{
    error::AUTDDriverError,
    firmware::{
        cpu::{RxMessage, TxMessage, check_if_msg_is_processed},
        operation::{Operation, OperationGenerator, OperationHandler},
    },
};

use itertools::Itertools;

/// The parallel processing mode.
#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelMode {
    /// Automatically select the processing mode. If the number of devices is greater than the parallel threshold of the [`Datagram::option`], the parallel processing is used.
    #[default]
    Auto = 0,
    /// Force to use the parallel processing.
    On = 1,
    /// Force to use the serial processing.
    Off = 2,
}

impl ParallelMode {
    #[must_use]
    pub(crate) const fn is_parallel(self, num_devices: usize, parallel_threshold: usize) -> bool {
        match self {
            ParallelMode::On => true,
            ParallelMode::Off => false,
            ParallelMode::Auto => num_devices > parallel_threshold,
        }
    }
}

/// The option of [`Sender`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SenderOption {
    /// The duration between sending operations.
    pub send_interval: Duration,
    /// The duration between receiving operations.
    pub receive_interval: Duration,
    /// If `None`, [`Datagram::option`] is used.
    ///
    /// [`Datagram`]: autd3_driver::datagram::Datagram
    pub timeout: Option<Duration>,
    /// The parallel processing mode.
    ///
    /// [`Datagram`]: autd3_driver::datagram::Datagram
    pub parallel: ParallelMode,
}

impl Default for SenderOption {
    fn default() -> Self {
        Self {
            send_interval: Duration::from_millis(1),
            receive_interval: Duration::from_millis(1),
            timeout: None,
            parallel: ParallelMode::Auto,
        }
    }
}

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: Link, S: Sleep> {
    pub(crate) msg_id: &'a mut MsgId,
    pub(crate) link: &'a mut L,
    pub(crate) geometry: &'a mut Geometry,
    pub(crate) sent_flags: &'a mut [bool],
    pub(crate) rx: &'a mut [RxMessage],
    pub(crate) option: SenderOption,
    pub(crate) sleeper: S,
}

impl<L: Link, S: Sleep> Sender<'_, L, S> {
    /// Send the [`Datagram`] to the devices.
    ///
    /// If the `timeout` value is
    /// - greater than 0, this function waits until the sent data is processed by the device or the specified timeout time elapses. If it cannot be confirmed that the sent data has been processed by the device, [`AUTDDriverError::ConfirmResponseFailed`] is returned.
    /// - 0, this function does not check whether the sent data has been processed by the device.
    ///
    /// The calculation of each [`Datagram`] is executed in parallel for each device if the number of devices is greater than the `parallel_threshold`.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
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

        let g = s.operation_generator(self.geometry, &DeviceFilter::all_enabled())?;
        let mut operations = OperationHandler::generate(g, self.geometry);
        operations.iter().enumerate().for_each(|(i, op)| {
            self.sent_flags[i] = op.is_some();
        });

        self.link.update(self.geometry)?;

        // We prioritize average behavior for the transmission timing. That is, not the interval from the previous transmission, but ensuring that T/`send_interval` transmissions are performed in a sufficiently long time T.
        // For example, if the `send_interval` is 1ms and it takes 1.5ms to transmit due to some reason, the next transmission will be performed not 1ms later but 0.5ms later.
        let mut send_timing = Instant::now();
        loop {
            let mut tx = self.link.alloc_tx_buffer()?;

            self.msg_id.increment();
            OperationHandler::pack(
                *self.msg_id,
                &mut operations,
                self.geometry,
                &mut tx,
                parallel,
            )?;

            self.send_receive(tx, timeout)?;

            if OperationHandler::is_done(&operations) {
                return Ok(());
            }

            send_timing += self.option.send_interval;
            self.sleeper.sleep_until(send_timing);
        }
    }

    fn send_receive(
        &mut self,
        tx: Vec<TxMessage>,
        timeout: Duration,
    ) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Err(AUTDDriverError::LinkClosed);
        }

        tracing::trace!("send: {}", tx.iter().join(", "));
        self.link.send(tx)?;
        self.wait_msg_processed(timeout)
    }

    fn wait_msg_processed(&mut self, timeout: Duration) -> Result<(), AUTDDriverError> {
        let start = Instant::now();
        let mut receive_timing = start;
        loop {
            if !self.link.is_open() {
                return Err(AUTDDriverError::LinkClosed);
            }
            self.link.receive(self.rx)?;
            tracing::trace!("send msg id: {}", self.msg_id.get());
            tracing::trace!("sent flags: {}", self.sent_flags.iter().join(", "));
            tracing::trace!("recv: {}", self.rx.iter().join(", "));

            if check_if_msg_is_processed(*self.msg_id, self.rx)
                .zip(self.sent_flags.iter())
                .filter_map(|(r, sent)| sent.then_some(r))
                .all(std::convert::identity)
            {
                return Ok(());
            }
            if start.elapsed() > timeout {
                break;
            }
            receive_timing += self.option.receive_interval;
            self.sleeper.sleep_until(receive_timing);
        }
        self.rx
            .iter()
            .try_fold((), |_, r| {
                autd3_driver::firmware::cpu::check_firmware_err(r)
            })
            .and_then(|_| {
                if timeout == Duration::ZERO {
                    Ok(())
                } else {
                    tracing::error!("Failed to confirm the response from the devices");
                    Err(AUTDDriverError::ConfirmResponseFailed)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::link::{LinkError, TxBufferPoolSync};

    use crate::{
        controller::sender::{SpinSleeper, StdSleeper},
        tests::create_geometry,
    };

    use super::*;

    #[rstest::rstest]
    #[case(true, ParallelMode::On, 1, 1)]
    #[case(true, ParallelMode::On, 2, 1)]
    #[case(true, ParallelMode::On, 1, 2)]
    #[case(false, ParallelMode::Off, 1, 1)]
    #[case(false, ParallelMode::Off, 2, 1)]
    #[case(false, ParallelMode::Off, 1, 2)]
    #[case(false, ParallelMode::Auto, 1, 1)]
    #[case(true, ParallelMode::Auto, 2, 1)]
    #[case(false, ParallelMode::Auto, 1, 2)]
    #[test]
    fn parallel_mode(
        #[case] expect: bool,
        #[case] mode: ParallelMode,
        #[case] num_devices: usize,
        #[case] threshold: usize,
    ) {
        assert_eq!(expect, mode.is_parallel(num_devices, threshold));
    }

    #[derive(Default)]
    struct MockLink {
        pub is_open: bool,
        pub send_cnt: usize,
        pub recv_cnt: usize,
        pub down: bool,
        pub buffer_pool: TxBufferPoolSync,
    }

    impl Link for MockLink {
        fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
            self.is_open = true;
            self.buffer_pool.init(geometry);
            Ok(())
        }

        fn close(&mut self) -> Result<(), LinkError> {
            self.is_open = false;
            Ok(())
        }

        fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
            Ok(self.buffer_pool.borrow())
        }

        fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
            if !self.down {
                self.send_cnt += 1;
            }
            self.buffer_pool.return_buffer(tx);
            Ok(())
        }

        fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
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

    #[test]
    fn test_close() -> anyhow::Result<()> {
        let mut link = MockLink::default();
        link.open(&Geometry::new(Vec::new()))?;

        assert!(link.is_open());

        link.close()?;

        assert!(!link.is_open());

        Ok(())
    }

    #[rstest::rstest]
    #[case(StdSleeper)]
    #[case(SpinSleeper::default())]
    #[test]
    fn test_send_receive(#[case] sleeper: impl Sleep) {
        let mut link = MockLink::default();
        let mut geometry = create_geometry(1);
        let mut sent_flags = vec![false; 1];
        let mut rx = Vec::new();
        let mut msg_id = MsgId::new(0);

        assert!(link.open(&geometry).is_ok());
        let mut sender = Sender {
            msg_id: &mut msg_id,
            link: &mut link,
            geometry: &mut geometry,
            sent_flags: &mut sent_flags,
            rx: &mut rx,
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel: ParallelMode::Auto,
            },
            sleeper,
        };

        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(Ok(()), sender.send_receive(tx, Duration::ZERO));

        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(Ok(()), sender.send_receive(tx, Duration::from_millis(1)));

        sender.link.is_open = false;
        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(
            sender.send_receive(tx, Duration::ZERO),
            Err(AUTDDriverError::LinkClosed)
        );
    }

    #[rstest::rstest]
    #[case(StdSleeper)]
    #[case(SpinSleeper::default())]
    #[test]
    fn test_wait_msg_processed(#[case] sleeper: impl Sleep) {
        let mut link = MockLink::default();
        let mut geometry = create_geometry(1);
        let mut sent_flags = vec![true; 1];
        let mut rx = vec![RxMessage::new(0, 0)];
        let mut msg_id = MsgId::new(1);

        assert!(link.open(&geometry).is_ok());
        let mut sender = Sender {
            msg_id: &mut msg_id,
            link: &mut link,
            geometry: &mut geometry,
            sent_flags: &mut sent_flags,
            rx: &mut rx,
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel: ParallelMode::Auto,
            },
            sleeper,
        };

        assert_eq!(Ok(()), sender.wait_msg_processed(Duration::from_millis(10)));

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
        assert_eq!(Ok(()), sender.wait_msg_processed(Duration::ZERO));

        sender.link.down = false;
        sender.link.recv_cnt = 0;
        *sender.msg_id = MsgId::new(20);
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("too many"))),
            sender.wait_msg_processed(Duration::from_secs(10))
        );
    }
}
