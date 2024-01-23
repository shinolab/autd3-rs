use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    datagram::{Clear, Synchronize},
    geometry::{Device, Geometry, IntoDevice},
};

use super::Controller;
use crate::error::AUTDError;

/// Builder for `Controller`
pub struct ControllerBuilder {
    devices: Vec<Device>,
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
        let mut cnt = Controller {
            link: link_builder.open(&geometry).await?,
            tx_buf: TxDatagram::new(geometry.num_devices()),
            rx_buf: vec![RxMessage { data: 0, ack: 0 }; geometry.num_devices()],
            geometry,
        };
        cnt.send(Clear::new()).await?;
        cnt.send(Synchronize::new()).await?;
        Ok(cnt)
    }
}
