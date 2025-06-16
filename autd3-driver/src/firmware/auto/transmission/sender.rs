use std::time::Duration;

use crate::{
    error::AUTDDriverError,
    firmware::{
        auto::{
            Version,
            operation::{Operation, OperationGenerator, OperationHandler},
        },
        driver::{SenderOption, TimerStrategy},
    },
};

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    derive::FirmwareLimits,
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
    pub(crate) option: SenderOption,
    pub(crate) timer_strategy: T,
    pub(crate) version: Version,
    pub(crate) limits: FirmwareLimits,
    pub(crate) _phantom: std::marker::PhantomData<S>,
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
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
        let strict = self.option.strict;

        let g = s.operation_generator(self.geometry, &DeviceFilter::all_enabled(), &self.limits)?;
        let mut operations = OperationHandler::generate(g, self.geometry, self.version);

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

            self.send_receive(tx, timeout, strict)?;

            if OperationHandler::is_done(&operations) {
                return Ok(());
            }

            send_timing = self
                .timer_strategy
                .sleep(send_timing, self.option.send_interval);
        }
    }
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    fn send_receive(
        &mut self,
        tx: Vec<TxMessage>,
        timeout: Duration,
        strict: bool,
    ) -> Result<(), AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.send(tx)?;
        match self.version {
            Version::V10 => crate::firmware::v10::transmission::Sender::wait_msg_processed(
                self.link,
                self.msg_id,
                self.rx,
                self.sent_flags,
                &self.option,
                &self.timer_strategy,
                timeout,
                strict,
            ),
            Version::V11 => crate::firmware::v11::transmission::Sender::wait_msg_processed(
                self.link,
                self.msg_id,
                self.rx,
                self.sent_flags,
                &self.option,
                &self.timer_strategy,
                timeout,
                strict,
            ),
            Version::V12 => crate::firmware::v12::transmission::Sender::wait_msg_processed(
                self.link,
                self.msg_id,
                self.rx,
                self.sent_flags,
                &self.option,
                &self.timer_strategy,
                timeout,
                strict,
            ),
        }
    }
}
