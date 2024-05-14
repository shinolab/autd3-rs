use autd3_driver::{
    datagram::{Clear, ConfigureFPGAClock, IntoDatagramWithTimeout, Synchronize},
    defined::{Freq, FREQ_40K},
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
    ultrasound_freq: Freq<u32>,
}

impl ControllerBuilder {
    pub(crate) const fn new_with_ultrasound_freq(ultrasound_freq: Freq<u32>) -> ControllerBuilder {
        Self {
            devices: vec![],
            ultrasound_freq,
        }
    }

    /// Add device
    pub fn add_device(mut self, dev: impl IntoDevice) -> Self {
        self.devices
            .push(dev.into_device(self.devices.len(), self.ultrasound_freq));
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
        let geometry = Geometry::new(self.devices, self.ultrasound_freq);
        let mut cnt = Controller {
            link: link_builder.open(&geometry).await?,
            tx_buf: TxDatagram::new(geometry.num_devices()),
            rx_buf: vec![RxMessage::new(0, 0); geometry.num_devices()],
            geometry,
        };
        if self.ultrasound_freq != FREQ_40K {
            cnt.send(ConfigureFPGAClock::new().with_timeout(timeout))
                .await?; // GRCOV_EXCL_LINE
        }
        cnt.send((Clear::new(), Synchronize::new()).with_timeout(timeout))
            .await?; // GRCOV_EXCL_LINE
        Ok(cnt)
    }
}
