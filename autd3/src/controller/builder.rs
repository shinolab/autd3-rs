use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    geometry::{Device, Geometry, IntoDevice},
    link::LinkBuilder,
};

use super::Controller;
use crate::error::AUTDError;

#[derive(Builder)]
pub struct ControllerBuilder {
    devices: Vec<Device>,
    #[get]
    #[set]
    parallel_threshold: usize,
    #[get]
    #[set]
    send_interval: std::time::Duration,
    #[cfg(target_os = "windows")]
    #[get]
    #[set]
    timer_resolution: std::num::NonZeroU32,
}

impl ControllerBuilder {
    pub(crate) fn new<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> ControllerBuilder {
        Self {
            devices: iter
                .into_iter()
                .enumerate()
                .map(|(i, d)| d.into_device(i))
                .collect(),
            parallel_threshold: 4,
            send_interval: std::time::Duration::from_millis(1),
            #[cfg(target_os = "windows")]
            timer_resolution: std::num::NonZeroU32::new(1).unwrap(),
        }
    }

    pub async fn open<B: LinkBuilder>(
        self,
        link_builder: B,
    ) -> Result<Controller<B::L>, AUTDError> {
        self.open_with_timeout(link_builder, DEFAULT_TIMEOUT).await
    }

    pub async fn open_with_timeout<B: LinkBuilder>(
        self,
        link_builder: B,
        timeout: std::time::Duration,
    ) -> Result<Controller<B::L>, AUTDError> {
        let geometry = Geometry::new(self.devices);
        let link = link_builder.open(&geometry).await?;
        let mut cnt = Controller {
            link,
            tx_buf: TxDatagram::new(geometry.num_devices()),
            rx_buf: vec![RxMessage::new(0, 0); geometry.num_devices()],
            geometry,
            parallel_threshold: self.parallel_threshold,
            last_parallel_threshold: self.parallel_threshold,
            send_interval: self.send_interval,
            #[cfg(target_os = "windows")]
            timer_resolution: self.timer_resolution,
        };
        cnt.open_impl(timeout).await?;
        Ok(cnt)
    }
}
