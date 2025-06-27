use std::time::{Duration, Instant};

use crate::{
    datagram::FirmwareVersionType,
    error::AUTDDriverError,
    firmware::{
        driver::{
            Operation, OperationHandler, SenderOption,
            r#async::{Driver, TimerStrategy},
        },
        v12::{V12, operation::OperationGenerator},
    },
};

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    geometry::Geometry,
    link::{AsyncLink, MsgId, RxMessage, TxMessage},
    sleep::r#async::Sleep,
};

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> {
    pub(crate) msg_id: &'a mut MsgId,
    pub(crate) link: &'a mut L,
    pub(crate) geometry: &'a Geometry,
    pub(crate) sent_flags: &'a mut [bool],
    pub(crate) rx: &'a mut [RxMessage],
    pub(crate) option: SenderOption,
    pub(crate) timer_strategy: T,
    pub(crate) _phantom: std::marker::PhantomData<S>,
}

impl<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    /// Send the [`Datagram`] to the devices.
    pub async fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        let timeout = self.option.timeout.unwrap_or(s.option().timeout);
        let parallel_threshold = s.option().parallel_threshold;

        let mut g = s.operation_generator(
            self.geometry,
            &DeviceFilter::all_enabled(),
            &V12.firmware_limits(),
        )?;
        let mut operations = self
            .geometry
            .iter()
            .map(|dev| g.generate(dev))
            .collect::<Vec<_>>();

        self.send_impl(timeout, parallel_threshold, &mut operations)
            .await
    }
}

impl<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    pub(crate) async fn send_impl<O1, O2>(
        &mut self,
        timeout: Duration,
        parallel_threshold: usize,
        operations: &mut [Option<(O1, O2)>],
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
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
        self.link.update(self.geometry).await?;

        let mut send_timing = self.timer_strategy.initial();
        loop {
            let mut tx = self.link.alloc_tx_buffer().await?;

            self.msg_id.increment();
            OperationHandler::pack(*self.msg_id, operations, self.geometry, &mut tx, parallel)?;

            self.send_receive(tx, timeout, strict).await?;

            if OperationHandler::is_done(operations) {
                return Ok(());
            }

            send_timing = self
                .timer_strategy
                .sleep(send_timing, self.option.send_interval)
                .await;
        }
    }

    async fn send_receive(
        &mut self,
        tx: Vec<TxMessage>,
        timeout: Duration,
        strict: bool,
    ) -> Result<(), AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.send(tx).await?;
        self.wait_msg_processed(timeout, strict).await
    }

    pub(crate) async fn wait_msg_processed(
        &mut self,
        timeout: Duration,
        strict: bool,
    ) -> Result<(), AUTDDriverError> {
        let start = Instant::now();
        let mut receive_timing = self.timer_strategy.initial();
        loop {
            self.link.ensure_is_open()?;
            self.link.receive(self.rx).await?;

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
                .sleep(receive_timing, self.option.receive_interval)
                .await;
        }

        self.rx.iter().try_fold((), |_, r| {
            crate::firmware::v12::cpu::check_firmware_err(r.ack())
        })
    }

    pub(crate) async fn fetch_firminfo(
        &mut self,
        ty: FirmwareVersionType,
    ) -> Result<Vec<u8>, AUTDDriverError> {
        self.send(ty).await.map_err(|_| {
            AUTDDriverError::ReadFirmwareVersionFailed(
                crate::firmware::v12::cpu::check_if_msg_is_processed(*self.msg_id, self.rx)
                    .collect(),
            )
        })?;
        Ok(self.rx.iter().map(|rx| rx.data()).collect())
    }
}
