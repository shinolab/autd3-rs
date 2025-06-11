use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::{
    error::AUTDDriverError,
    firmware::{
        cpu::{RxMessage, TxMessage, check_if_msg_is_processed},
        operation::{Operation, OperationGenerator, OperationHandler},
        version::FirmwareVersion,
    },
};
use autd3_core::{
    datagram::Datagram,
    derive::DeviceFilter,
    geometry::Geometry,
    link::{Link, MsgId},
    sleep::Sleep,
};

pub use super::parallel_mode::ParallelMode;
pub use super::strategy::TimerStrategy;

/// The option of [`Sender`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SenderOption {
    /// The duration between sending operations.
    pub send_interval: Duration,
    /// The duration between receiving operations.
    pub receive_interval: Duration,
    /// If `None`, [`Datagram::option`] is used.
    ///
    /// [`Datagram`]: crate::datagram::Datagram
    pub timeout: Option<Duration>,
    /// The parallel processing mode.
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
pub struct Sender<'a, L: Link, S: Sleep, T: TimerStrategy<S>> {
    pub(crate) msg_id: &'a mut MsgId,
    pub(crate) link: &'a mut L,
    pub(crate) geometry: &'a mut Geometry,
    pub(crate) sent_flags: &'a mut [bool],
    pub(crate) rx: &'a mut [RxMessage],
    pub(crate) option: SenderOption,
    pub(crate) timer_strategy: T,
    pub(crate) _phantom: std::marker::PhantomData<S>,
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    #[doc(hidden)]
    pub fn new(
        msg_id: &'a mut MsgId,
        link: &'a mut L,
        geometry: &'a mut Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [RxMessage],
        option: SenderOption,
        timer_strategy: T,
    ) -> Self {
        Self {
            msg_id,
            link,
            geometry,
            sent_flags,
            rx,
            option,
            timer_strategy,
            _phantom: std::marker::PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn initialize_devices(&mut self) -> Result<(), AUTDDriverError> {
        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data (here, we use `ReadsFPGAState` because it is the lightest).
        let _ = self.send(crate::datagram::ReadsFPGAState::new(|_| false));

        self.send((
            crate::datagram::Clear::new(),
            crate::datagram::Synchronize::new(),
        ))
    }

    /// Send the [`Datagram`] to the devices.
    ///
    /// If the `timeout` value is
    /// - greater than 0, this function waits until the sent data is processed by the device or the specified timeout time elapses. If it cannot be confirmed that the sent data has been processed by the device, [`AUTDDriverError::ConfirmResponseFailed`] is returned.
    /// - 0, this function does not check whether the sent data has been processed by the device.
    ///
    /// The calculation of each [`Datagram`] is executed in parallel for each device if the number of devices is greater than the `parallel_threshold`.
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        let timeout = self.option.timeout.unwrap_or(s.option().timeout);
        let parallel_threshold = s.option().parallel_threshold;

        let g = s.operation_generator(self.geometry, &DeviceFilter::all_enabled())?;
        let mut operations = OperationHandler::generate(g, self.geometry);

        operations
            .iter()
            .zip(self.sent_flags.iter_mut())
            .for_each(|(op, flag)| {
                *flag = op.is_some();
            });

        let num_enabled = self.sent_flags.iter().filter(|x| **x).count();
        let parallel = self
            .option
            .parallel
            .is_parallel(num_enabled, parallel_threshold);

        self.link.ensure_is_open()?;
        self.link.update(self.geometry)?;

        let mut send_timing = T::initial();
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

            send_timing = self
                .timer_strategy
                .sleep(send_timing, self.option.send_interval);
        }
    }

    #[doc(hidden)]
    pub fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDDriverError> {
        use crate::firmware::{
            operation::FirmwareVersionType::*,
            version::{CPUVersion, FPGAVersion, Major, Minor},
        };

        let cpu_major = self.fetch_firminfo(CPUMajor)?;
        let cpu_minor = self.fetch_firminfo(CPUMinor)?;
        let fpga_major = self.fetch_firminfo(FPGAMajor)?;
        let fpga_minor = self.fetch_firminfo(FPGAMinor)?;
        let fpga_functions = self.fetch_firminfo(FPGAFunctions)?;
        self.fetch_firminfo(Clear)?;

        Ok(self
            .geometry
            .iter()
            .map(|dev| FirmwareVersion {
                idx: dev.idx(),
                cpu: CPUVersion {
                    major: Major(cpu_major[dev.idx()]),
                    minor: Minor(cpu_minor[dev.idx()]),
                },
                fpga: FPGAVersion {
                    major: Major(fpga_major[dev.idx()]),
                    minor: Minor(fpga_minor[dev.idx()]),
                    function_bits: fpga_functions[dev.idx()],
                },
            })
            .collect())
    }
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    fn send_receive(
        &mut self,
        tx: Vec<TxMessage>,
        timeout: Duration,
    ) -> Result<(), AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.send(tx)?;
        self.wait_msg_processed(timeout)
    }

    fn wait_msg_processed(&mut self, timeout: Duration) -> Result<(), AUTDDriverError> {
        let start = Instant::now();
        let mut receive_timing = T::initial();
        loop {
            self.link.ensure_is_open()?;
            self.link.receive(self.rx)?;

            if check_if_msg_is_processed(*self.msg_id, self.rx)
                .zip(self.sent_flags.iter())
                .filter_map(|(r, sent)| sent.then_some(r))
                .all(std::convert::identity)
            {
                break;
            }

            if start.elapsed() > timeout {
                return if timeout == Duration::ZERO {
                    Ok(())
                } else {
                    Err(AUTDDriverError::ConfirmResponseFailed)
                };
            }

            receive_timing = self
                .timer_strategy
                .sleep(receive_timing, self.option.receive_interval);
        }

        self.rx
            .iter()
            .try_fold((), |_, r| AUTDDriverError::check_firmware_err(r.ack()))
    }

    fn fetch_firminfo(
        &mut self,
        ty: crate::firmware::operation::FirmwareVersionType,
    ) -> Result<Vec<u8>, AUTDDriverError> {
        self.send(ty).map_err(|_| {
            AUTDDriverError::ReadFirmwareVersionFailed(
                crate::firmware::cpu::check_if_msg_is_processed(*self.msg_id, self.rx).collect(),
            )
        })?;
        Ok(self.rx.iter().map(|rx| rx.data()).collect())
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::{
        link::{Ack, LinkError, TxBufferPoolSync},
        sleep::{Sleep, SpinSleeper, SpinWaitSleeper, StdSleeper},
    };

    use crate::datagram::tests::create_geometry;

    use super::{super::strategy::FixedSchedule, *};

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
            rx.iter_mut().for_each(|r| {
                *r = RxMessage::new(r.data(), Ack::new().with_msg_id(self.recv_cnt as u8))
            });

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
    #[case(SpinWaitSleeper)]
    #[test]
    fn test_send_receive(#[case] sleeper: impl Sleep) {
        let mut link = MockLink::default();
        let mut geometry = create_geometry(1, 1);
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
            timer_strategy: FixedSchedule(sleeper),
            _phantom: std::marker::PhantomData,
        };

        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(Ok(()), sender.send_receive(tx, Duration::ZERO));

        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(Ok(()), sender.send_receive(tx, Duration::from_millis(1)));

        sender.link.is_open = false;
        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
            sender.send_receive(tx, Duration::ZERO),
        );
    }

    #[rstest::rstest]
    #[case(StdSleeper)]
    #[case(SpinSleeper::default())]
    #[case(SpinWaitSleeper)]
    #[test]
    fn test_wait_msg_processed(#[case] sleeper: impl Sleep) {
        let mut link = MockLink::default();
        let mut geometry = create_geometry(1, 1);
        let mut sent_flags = vec![true; 1];
        let mut rx = vec![RxMessage::new(0, Ack::new())];
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
            timer_strategy: FixedSchedule(sleeper),
            _phantom: std::marker::PhantomData,
        };

        assert_eq!(Ok(()), sender.wait_msg_processed(Duration::from_millis(10)));

        sender.link.recv_cnt = 0;
        sender.link.is_open = false;
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
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
