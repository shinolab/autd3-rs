mod builder;
mod group;

use std::{hash::Hash, time::Duration};

use autd3_driver::{
    datagram::{Clear, ConfigureSilencer, Datagram},
    defined::DEFAULT_TIMEOUT,
    firmware::{
        cpu::{RxMessage, TxDatagram},
        fpga::FPGAState,
        operation::OperationHandler,
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry},
    link::{send_receive, Link},
};

use crate::{
    error::{AUTDError, ReadFirmwareVersionState},
    gain::Null,
    link::nop::Nop,
};

pub use builder::ControllerBuilder;
pub use group::GroupGuard;

/// Controller for AUTD
pub struct Controller<L: Link> {
    pub link: L,
    pub geometry: Geometry,
    tx_buf: TxDatagram,
    rx_buf: Vec<RxMessage>,
}

impl Controller<Nop> {
    /// Create Controller builder
    pub const fn builder() -> ControllerBuilder {
        ControllerBuilder::new()
    }
}

impl<L: Link> Controller<L> {
    #[must_use]
    pub fn group<K: Hash + Eq + Clone, F: Fn(&Device) -> Option<K>>(
        &mut self,
        f: F,
    ) -> GroupGuard<K, L, F> {
        GroupGuard::new(self, f)
    }
}

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
    pub async fn send(&mut self, s: impl Datagram) -> Result<bool, AUTDError> {
        let timeout = s.timeout();

        let (mut op1, mut op2) = s.operation()?;
        OperationHandler::init(&mut op1, &mut op2, &self.geometry)?;
        loop {
            OperationHandler::pack(&mut op1, &mut op2, &self.geometry, &mut self.tx_buf)?;

            let start = tokio::time::Instant::now();
            if !send_receive(&mut self.link, &self.tx_buf, &mut self.rx_buf, timeout).await? {
                return Ok(false);
            }
            if OperationHandler::is_finished(&mut op1, &mut op2, &self.geometry) {
                return Ok(true);
            }
            tokio::time::sleep_until(start + Duration::from_millis(1)).await;
        }
    }

    // Close connection
    pub async fn close(&mut self) -> Result<bool, AUTDError> {
        if !self.link.is_open() {
            return Ok(true);
        }
        self.geometry.iter_mut().for_each(|dev| dev.enable = true);
        let res = self
            .send((Null::default(), ConfigureSilencer::default()))
            .await?
            & self.send(Clear::new()).await?;
        self.link.close().await?;
        Ok(res)
    }

    /// Get firmware information
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<FirmwareVersion>)` - List of firmware information
    ///
    pub async fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDError> {
        let mut op = autd3_driver::firmware::operation::FirmInfoOp::default();
        let mut null_op = autd3_driver::firmware::operation::NullOp::default();

        OperationHandler::init(&mut op, &mut null_op, &self.geometry)?;

        macro_rules! pack_and_send {
            ($op:expr, $null_op:expr, $link:expr, $geometry:expr, $tx_buf:expr, $rx_buf:expr ) => {
                OperationHandler::pack($op, $null_op, $geometry, $tx_buf)?;
                if !autd3_driver::link::send_receive($link, $tx_buf, $rx_buf, Some(DEFAULT_TIMEOUT))
                    .await?
                {
                    return Err(AUTDError::ReadFirmwareVersionFailed(
                        ReadFirmwareVersionState(
                            autd3_driver::firmware::cpu::check_if_msg_is_processed(
                                $tx_buf, $rx_buf,
                            )
                            .collect(),
                        ),
                    ));
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
        let cpu_versions = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let cpu_versions_minor = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_versions = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_versions_minor = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut op,
            &mut null_op,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf
        );
        let fpga_functions = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

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
                FirmwareVersion::new(
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

impl<L: Link> Drop for Controller<L> {
    fn drop(&mut self) {
        if !self.link.is_open() {
            return;
        }
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

#[cfg(test)]
mod tests {
    use autd3_driver::{autd3_device::AUTD3, geometry::Vector3};

    use crate::link::Audit;

    use super::*;

    pub async fn create_controller(dev_num: usize) -> anyhow::Result<Controller<Audit>> {
        Ok((0..dev_num)
            .fold(Controller::builder(), |acc, _i| {
                acc.add_device(AUTD3::new(Vector3::zeros()))
            })
            .open(Audit::builder())
            .await?)
    }
}
