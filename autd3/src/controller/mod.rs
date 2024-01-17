/*
 * File: mod.rs
 * Project: controller
 * Created Date: 05/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

pub mod builder;
mod group;

use std::{collections::HashMap, hash::Hash, time::Duration};

use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    datagram::{Clear, Datagram},
    firmware_version::FirmwareInfo,
    fpga::FPGAState,
    geometry::{Device, Geometry},
    operation::OperationHandler,
};

use crate::error::{AUTDError, ReadFirmwareInfoState};

pub use builder::ControllerBuilder;
pub use group::GroupGuard;

use crate::link::nop::Nop;

#[cfg(not(feature = "sync"))]
use autd3_driver::link::Link;
#[cfg(feature = "sync")]
use autd3_driver::link::LinkSync as Link;

/// Controller for AUTD
pub struct Controller<L: Link> {
    pub link: L,
    pub geometry: Geometry,
    tx_buf: TxDatagram,
    rx_buf: Vec<RxMessage>,
    ignore_ack: bool,
}

impl Controller<Nop> {
    /// Create Controller builder
    pub const fn builder() -> ControllerBuilder {
        ControllerBuilder::new()
    }

    /// Create Controller builder
    pub const fn builder_with() -> ControllerBuilder {
        ControllerBuilder::new()
    }
}

impl<L: Link> Controller<L> {
    #[must_use]
    pub fn group<K: Hash + Eq + Clone, F: Fn(&Device) -> Option<K>>(
        &mut self,
        f: F,
    ) -> GroupGuard<K, L, F> {
        GroupGuard {
            cnt: self,
            f,
            timeout: None,
            op: HashMap::new(),
        }
    }
}

#[cfg(not(feature = "sync"))]
impl<L: Link> Controller<L> {
    /// Send data to the devices
    ///
    /// # Arguments
    ///
    /// * `s` - Datagram
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - It is confirmed that the data has been successfully transmitted
    /// * `Ok(false)` - There are no errors, but it is unclear whether the data has been sent or not
    ///
    pub async fn send<S: Datagram>(&mut self, s: S) -> Result<bool, AUTDError> {
        let timeout = s.timeout();

        let (mut op1, mut op2) = s.operation()?;
        OperationHandler::init(&mut op1, &mut op2, &self.geometry)?;
        loop {
            let start = std::time::Instant::now();
            OperationHandler::pack(&mut op1, &mut op2, &self.geometry, &mut self.tx_buf)?;

            if !self
                .link
                .send_receive(&self.tx_buf, &mut self.rx_buf, timeout, self.ignore_ack)
                .await?
            {
                return Ok(false);
            }
            if OperationHandler::is_finished(&mut op1, &mut op2, &self.geometry) {
                break;
            }
            if start.elapsed() < std::time::Duration::from_millis(1) {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        Ok(true)
    }

    // Close connection
    pub async fn close(&mut self) -> Result<bool, AUTDError> {
        if !self.link.is_open() {
            return Ok(false);
        }
        for dev in self.geometry.iter_mut() {
            dev.enable = true;
        }
        self.ignore_ack = true;
        let res = self
            .send((
                crate::gain::Null::default(),
                autd3_driver::datagram::ConfigureSilencer::default(),
            ))
            .await?;
        let res = res & self.send(Clear::new()).await?;
        self.link.close().await?;
        Ok(res)
    }

    /// Get firmware information
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<FirmwareInfo>)` - List of firmware information
    ///
    pub async fn firmware_infos(&mut self) -> Result<Vec<FirmwareInfo>, AUTDError> {
        let mut op = autd3_driver::operation::FirmInfoOp::default();
        let mut null_op = autd3_driver::operation::NullOp::default();

        OperationHandler::init(&mut op, &mut null_op, &self.geometry)?;

        macro_rules! pack_and_send {
            ($op:expr, $null_op:expr, $link:expr, $geometry:expr, $tx_buf:expr, $rx_buf:expr ) => {
                OperationHandler::pack($op, $null_op, $geometry, $tx_buf)?;
                if !$link
                    .send_receive($tx_buf, $rx_buf, Some(Duration::from_millis(200)), false)
                    .await?
                {
                    return Err(AUTDError::ReadFirmwareInfoFailed(ReadFirmwareInfoState(
                        autd3_driver::cpu::check_if_msg_is_processed($tx_buf, $rx_buf),
                    )));
                }
            };
        }

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let cpu_versions = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let cpu_versions_minor = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_versions = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_versions_minor = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_functions = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );

        Ok((0..self.geometry.num_devices())
            .map(|i| {
                FirmwareInfo::new(
                    i,
                    cpu_versions[i],
                    cpu_versions_minor[i],
                    fpga_versions[i],
                    fpga_versions_minor[i],
                    fpga_functions[i],
                )
            })
            .collect())
    }

    /// Get FPGA state
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Option<FPGAState>>)` - List of FPGA state the latest data is fetched. If the reads FPGA state flag is not set, the value is None. See [autd3_driver::datagram::ConfigureReadsFPGAState].
    /// * `Err(AUTDError::ReadFPGAStateFailed)` - If failure to fetch the latest data
    ///
    pub async fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDError> {
        if self.link.receive(&mut self.rx_buf).await? {
            Ok(self.rx_buf.iter().map(Option::<FPGAState>::from).collect())
        } else {
            Err(AUTDError::ReadFPGAStateFailed)
        }
    }
}

#[cfg(feature = "sync")]
impl<L: Link> Controller<L> {
    pub fn send<S: Datagram>(&mut self, s: S) -> Result<bool, AUTDError> {
        let timeout = s.timeout();

        let (mut op1, mut op2) = s.operation()?;
        OperationHandler::init(&mut op1, &mut op2, &self.geometry)?;
        loop {
            let start = std::time::Instant::now();
            OperationHandler::pack(&mut op1, &mut op2, &self.geometry, &mut self.tx_buf)?;

            if !self
                .link
                .send_receive(&self.tx_buf, &mut self.rx_buf, timeout, self.ignore_ack)?
            {
                return Ok(false);
            }
            if OperationHandler::is_finished(&mut op1, &mut op2, &self.geometry) {
                break;
            }
            if start.elapsed() < std::time::Duration::from_millis(1) {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(true)
    }

    pub fn close(&mut self) -> Result<bool, AUTDError> {
        if !self.link.is_open() {
            return Ok(false);
        }
        for dev in self.geometry.iter_mut() {
            dev.enable = true;
        }
        let res = self.send((
            autd3_driver::datagram::ConfigureSilencer::default(),
            crate::gain::Null::default(),
        ))?;
        let res = res & self.send(Clear::new())?;
        self.link.close()?;
        Ok(res)
    }

    pub fn firmware_infos(&mut self) -> Result<Vec<FirmwareInfo>, AUTDError> {
        let mut op = autd3_driver::operation::FirmInfoOp::default();
        let mut null_op = autd3_driver::operation::NullOp::default();

        OperationHandler::init(&mut op, &mut null_op, &self.geometry)?;

        macro_rules! pack_and_send {
            ($op:expr, $null_op:expr, $link:expr, $geometry:expr, $tx_buf:expr, $rx_buf:expr ) => {
                OperationHandler::pack($op, $null_op, $geometry, $tx_buf)?;
                if !$link.send_receive($tx_buf, $rx_buf, Some(Duration::from_millis(200)), false)? {
                    return Err(AUTDError::ReadFirmwareInfoFailed(ReadFirmwareInfoState(
                        autd3_driver::cpu::check_if_msg_is_processed($tx_buf, $rx_buf),
                    )));
                }
            };
        }

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let cpu_versions = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let cpu_versions_minor = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_versions = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_versions_minor = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_functions = self.rx_buf.iter().map(|rx| rx.data).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );

        Ok((0..self.geometry.num_devices())
            .map(|i| {
                FirmwareInfo::new(
                    i,
                    cpu_versions[i],
                    cpu_versions_minor[i],
                    fpga_versions[i],
                    fpga_versions_minor[i],
                    fpga_functions[i],
                )
            })
            .collect())
    }

    pub fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDError> {
        if self.link.receive(&mut self.rx_buf).await? {
            Ok(self.rx_buf.iter().map(Option::<FPGAState>::from).collect())
        } else {
            Err(AUTDError::ReadFPGAStateFailed)
        }
    }
}

#[cfg(not(feature = "sync"))]
impl<L: Link> Drop for Controller<L> {
    fn drop(&mut self) {
        match tokio::runtime::Handle::current().runtime_flavor() {
            tokio::runtime::RuntimeFlavor::CurrentThread => {}
            tokio::runtime::RuntimeFlavor::MultiThread => tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = self.close().await;
                });
            }),
            _ => unimplemented!(),
        }
    }
}

#[cfg(feature = "sync")]
impl<L: Link> Drop for Controller<L> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
