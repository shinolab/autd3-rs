/*
 * File: builder.rs
 * Project: controller
 * Created Date: 05/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    datagram::{Clear, DatagramT, Synchronize},
    geometry::{Device, Geometry, IntoDevice},
};

use super::Controller;
use crate::error::AUTDError;

/// Builder for `Controller`
pub struct ControllerBuilder {
    devices: Vec<Device>,
}

impl Default for ControllerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ControllerBuilder {
    pub(crate) const fn new() -> ControllerBuilder {
        Self { devices: vec![] }
    }

    /// Add device
    pub fn add_device<D: IntoDevice>(mut self, dev: D) -> Self {
        self.devices.push(dev.into_device(self.devices.len()));
        self
    }

    /// Open controller
    pub async fn open_with<B: autd3_driver::link::LinkBuilder>(
        self,
        link_builder: B,
    ) -> Result<Controller<B::L>, AUTDError> {
        let geometry = Geometry::new(self.devices);
        let link = link_builder.open(&geometry).await?;

        let num_devices = geometry.num_devices();
        let tx_buf = TxDatagram::new(num_devices);
        let mut cnt = Controller {
            link,
            geometry,
            tx_buf,
            rx_buf: vec![RxMessage { data: 0, ack: 0 }; num_devices],
        };
        cnt.send(Clear::new().with_timeout(std::time::Duration::ZERO))
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        cnt.send(Clear::new()).await?;
        cnt.send(Synchronize::new()).await?;
        Ok(cnt)
    }
}
