use std::time::{Duration, Instant};

use crate::{
    datagram::FirmwareVersionType,
    error::AUTDDriverError,
    firmware::{
        driver::{Driver, Operation, OperationHandler, SenderOption, TimerStrategy},
        v12::{V12, operation::OperationGenerator},
    },
};

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    geometry::Geometry,
    link::{Link, MsgId, RxMessage, TxMessage},
    sleep::Sleep,
};

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: Link, S: Sleep, T: TimerStrategy<S>> {
    pub(crate) msg_id: &'a mut MsgId,
    pub(crate) link: &'a mut L,
    pub(crate) geometry: &'a Geometry,
    pub(crate) sent_flags: &'a mut [bool],
    pub(crate) rx: &'a mut [RxMessage],
    pub(crate) env: &'a Environment,
    pub(crate) option: SenderOption,
    pub(crate) timer_strategy: T,
    pub(crate) _phantom: std::marker::PhantomData<S>,
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    /// Send the [`Datagram`] to the devices.
    pub fn send<D: Datagram<'a>>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as OperationGenerator<'a>>::O1 as Operation<'a>>::Error>
            + From<<<D::G as OperationGenerator<'a>>::O2 as Operation<'a>>::Error>,
    {
        let timeout = self.option.timeout.unwrap_or(s.option().timeout);
        let parallel_threshold = s.option().parallel_threshold;

        let mut g = s.operation_generator(
            self.geometry,
            self.env,
            &DeviceMask::AllEnabled,
            &V12.firmware_limits(),
        )?;
        let mut operations = self
            .geometry
            .iter()
            .map(|dev| g.generate(dev))
            .collect::<Vec<_>>();

        self.send_impl(timeout, parallel_threshold, &mut operations)
    }
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    pub(crate) fn send_impl<O1, O2>(
        &mut self,
        timeout: Duration,
        parallel_threshold: usize,
        operations: &mut [Option<(O1, O2)>],
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation<'a>,
        O2: Operation<'a>,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        let strict = self.option.strict;

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

        let mut send_timing = self.timer_strategy.initial();
        loop {
            let mut tx = self.link.alloc_tx_buffer()?;

            self.msg_id.increment();
            OperationHandler::pack(*self.msg_id, operations, self.geometry, &mut tx, parallel)?;

            self.send_receive(tx, timeout, strict)?;

            if OperationHandler::is_done(operations) {
                return Ok(());
            }

            send_timing = self
                .timer_strategy
                .sleep(send_timing, self.option.send_interval);
        }
    }

    fn send_receive(
        &mut self,
        tx: Vec<TxMessage>,
        timeout: Duration,
        strict: bool,
    ) -> Result<(), AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.send(tx)?;
        self.wait_msg_processed(timeout, strict)
    }

    fn wait_msg_processed(
        &mut self,
        timeout: Duration,
        strict: bool,
    ) -> Result<(), AUTDDriverError> {
        let start = Instant::now();
        let mut receive_timing = self.timer_strategy.initial();
        loop {
            self.link.ensure_is_open()?;
            self.link.receive(self.rx)?;

            if crate::firmware::v12::cpu::check_if_msg_is_processed(*self.msg_id, self.rx)
                .zip(self.sent_flags.iter())
                .filter_map(|(r, sent)| sent.then_some(r))
                .all(std::convert::identity)
            {
                break;
            }

            if start.elapsed() > timeout {
                return if !strict && timeout == Duration::ZERO {
                    Ok(())
                } else {
                    Err(AUTDDriverError::ConfirmResponseFailed)
                };
            }

            receive_timing = self
                .timer_strategy
                .sleep(receive_timing, self.option.receive_interval);
        }

        self.rx.iter().try_fold((), |_, r| {
            crate::firmware::v12::cpu::check_firmware_err(r.ack())
        })
    }

    pub(crate) fn fetch_firminfo(
        &mut self,
        ty: FirmwareVersionType,
    ) -> Result<Vec<u8>, AUTDDriverError> {
        self.send(ty).map_err(|_| {
            AUTDDriverError::ReadFirmwareVersionFailed(
                crate::firmware::v12::cpu::check_if_msg_is_processed(*self.msg_id, self.rx)
                    .collect(),
            )
        })?;
        Ok(self.rx.iter().map(|rx| rx.data()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::firmware::driver::{FixedSchedule, ParallelMode};
    use autd3_core::{
        link::{Ack, LinkError, TxBufferPoolSync},
        sleep::{Sleep, SpinSleeper, SpinWaitSleeper, StdSleeper},
    };

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
    fn test_close() -> Result<(), Box<dyn std::error::Error>> {
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
        let mut geometry = crate::autd3_device::tests::create_geometry(1);
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
            env: &Environment::default(),
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel: ParallelMode::Auto,
                strict: true,
            },
            timer_strategy: FixedSchedule(sleeper),
            _phantom: std::marker::PhantomData,
        };

        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(Ok(()), sender.send_receive(tx, Duration::ZERO, true));

        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(
            Ok(()),
            sender.send_receive(tx, Duration::from_millis(1), true)
        );

        sender.link.is_open = false;
        let tx = sender.link.alloc_tx_buffer().unwrap();
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
            sender.send_receive(tx, Duration::ZERO, true),
        );
    }

    #[rstest::rstest]
    #[case(StdSleeper)]
    #[case(SpinSleeper::default())]
    #[case(SpinWaitSleeper)]
    #[test]
    fn test_wait_msg_processed(#[case] sleeper: impl Sleep) {
        let mut link = MockLink::default();
        let mut geometry = crate::autd3_device::tests::create_geometry(1);
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
            env: &Environment::default(),
            option: SenderOption {
                send_interval: Duration::from_millis(1),
                receive_interval: Duration::from_millis(1),
                timeout: None,
                parallel: ParallelMode::Auto,
                strict: true,
            },
            timer_strategy: FixedSchedule(sleeper),
            _phantom: std::marker::PhantomData,
        };

        assert_eq!(
            Ok(()),
            sender.wait_msg_processed(Duration::from_millis(10), true)
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = false;
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
            sender.wait_msg_processed(Duration::from_millis(10), true)
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(
            Err(AUTDDriverError::ConfirmResponseFailed),
            sender.wait_msg_processed(Duration::from_millis(10), true)
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(
            Err(AUTDDriverError::ConfirmResponseFailed),
            sender.wait_msg_processed(Duration::ZERO, true)
        );

        sender.link.recv_cnt = 0;
        sender.link.is_open = true;
        sender.link.down = true;
        assert_eq!(Ok(()), sender.wait_msg_processed(Duration::ZERO, false));

        sender.link.down = false;
        sender.link.recv_cnt = 0;
        *sender.msg_id = MsgId::new(20);
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("too many"))),
            sender.wait_msg_processed(Duration::from_secs(10), true)
        );
    }
}
