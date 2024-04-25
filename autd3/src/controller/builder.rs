use autd3_driver::{
    datagram::{Clear, IntoDatagramWithTimeout, Synchronize},
    derive::DEFAULT_TIMEOUT,
    firmware::cpu::{RxMessage, TxDatagram},
    geometry::{Device, Geometry, IntoDevice},
    link::LinkBuilder,
};

use super::Controller;
use crate::error::AUTDError;

/// Builder for [crate::controller::Controller]
pub struct ControllerBuilder {
    devices: Vec<Device>,
}

impl ControllerBuilder {
    pub(crate) const fn new() -> ControllerBuilder {
        Self { devices: vec![] }
    }

    /// Add device
    pub fn add_device(mut self, dev: impl IntoDevice) -> Self {
        self.devices.push(dev.into_device(self.devices.len()));
        self
    }

    /// Open controller
    pub async fn open<B: LinkBuilder>(
        self,
        link_builder: B,
    ) -> Result<Controller<B::L>, AUTDError> {
        self.open_with_timeout(link_builder, DEFAULT_TIMEOUT).await
    }

    /// Open controller with timeout
    pub async fn open_with_timeout<B: LinkBuilder>(
        self,
        link_builder: B,
        timeout: std::time::Duration,
    ) -> Result<Controller<B::L>, AUTDError> {
        let geometry = Geometry::new(self.devices);
        let mut cnt = Controller {
            link: link_builder.open(&geometry).await?,
            tx_buf: TxDatagram::new(geometry.num_devices()),
            rx_buf: vec![RxMessage::new(0, 0); geometry.num_devices()],
            geometry,
        };
        cnt.send((Clear::new(), Synchronize::new()).with_timeout(timeout))
            .await?; // GRCOV_EXCL_LINE
        Ok(cnt)
    }
}
