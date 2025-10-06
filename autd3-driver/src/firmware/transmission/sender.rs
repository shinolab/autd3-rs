use std::time::{Duration, Instant};

use super::SenderOption;
use crate::{
    datagram::{
        Clear, FirmwareVersionType, FixedCompletionSteps, Silencer,
        implements::{Null, Static},
    },
    error::AUTDDriverError,
    firmware::{
        operation::{Operation, OperationGenerator, OperationHandler},
        version::FirmwareVersion,
    },
};

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    geometry::Geometry,
    link::{Link, MsgId, RxMessage, TxMessage},
    sleep::Sleeper,
};

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: Link, S: Sleeper> {
    pub(crate) msg_id: &'a mut MsgId,
    pub(crate) link: &'a mut L,
    pub(crate) geometry: &'a Geometry,
    pub(crate) sent_flags: &'a mut [bool],
    pub(crate) rx: &'a mut [RxMessage],
    pub(crate) env: &'a Environment,
    pub(crate) option: SenderOption,
    pub(crate) sleeper: S,
}

impl<'a, L: Link, S: Sleeper> Sender<'a, L, S> {
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::geometry::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
        env: &'a Environment,
        option: SenderOption,
        sleeper: S,
    ) -> Self {
        Self {
            msg_id,
            link,
            geometry,
            sent_flags,
            rx,
            env,
            option,
            sleeper,
        }
    }

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

        let mut g = s.operation_generator(self.geometry, self.env, &DeviceMask::AllEnabled)?;
        let mut operations = self
            .geometry
            .iter()
            .map(|dev| g.generate(dev))
            .collect::<Vec<_>>();

        self.send_impl(timeout, parallel_threshold, &mut operations)
    }

    #[doc(hidden)]
    pub fn initialize_devices(mut self) -> Result<(), crate::error::AUTDDriverError> {
        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data.
        self.send(crate::datagram::Nop)?;

        self.send((Clear::new(), crate::datagram::Synchronize::new()))
    }

    #[doc(hidden)]
    pub fn firmware_version(
        mut self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        use crate::{
            datagram::FirmwareVersionType::*,
            firmware::version::{CPUVersion, FPGAVersion, Major, Minor},
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

    #[doc(hidden)]
    pub fn close(mut self) -> Result<(), crate::error::AUTDDriverError> {
        [
            self.send(Silencer {
                config: FixedCompletionSteps {
                    strict: false,
                    ..Default::default()
                },
            }),
            self.send((Static::default(), Null)),
            self.send(Clear {}),
            Ok(self.link.close()?),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }
}

impl<'a, L: Link, S: Sleeper> Sender<'a, L, S> {
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

        let mut send_timing = Instant::now();
        loop {
            let mut tx = self.link.alloc_tx_buffer()?;

            self.msg_id.increment();
            OperationHandler::pack(*self.msg_id, operations, self.geometry, &mut tx, parallel)?;

            self.send_receive(tx, timeout)?;

            if OperationHandler::is_done(operations) {
                return Ok(());
            }

            let next = send_timing + self.option.send_interval;
            self.sleeper
                .sleep(next.saturating_duration_since(Instant::now()));
            send_timing = next;
        }
    }

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
        let mut receive_timing = Instant::now();
        loop {
            self.link.ensure_is_open()?;
            self.link.receive(self.rx)?;

            if crate::firmware::cpu::check_if_msg_is_processed(*self.msg_id, self.rx)
                .zip(self.sent_flags.iter())
                .filter_map(|(r, sent)| sent.then_some(r))
                .all(std::convert::identity)
            {
                break;
            }

            if start.elapsed() > timeout {
                return Err(AUTDDriverError::ConfirmResponseFailed);
            }

            let next = receive_timing + self.option.receive_interval;
            self.sleeper
                .sleep(next.saturating_duration_since(Instant::now()));
            receive_timing = next;
        }

        self.rx
            .iter()
            .try_fold((), |_, r| crate::firmware::cpu::check_firmware_err(r.ack()))
    }

    pub(crate) fn fetch_firminfo(
        &mut self,
        ty: FirmwareVersionType,
    ) -> Result<Vec<u8>, AUTDDriverError> {
        self.send(ty).map_err(|_| {
            AUTDDriverError::ReadFirmwareVersionFailed(
                crate::firmware::cpu::check_if_msg_is_processed(*self.msg_id, self.rx).collect(),
            )
        })?;
        Ok(self.rx.iter().map(|rx| rx.data()).collect())
    }
}
