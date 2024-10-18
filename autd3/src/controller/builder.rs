use std::time::Duration;

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    geometry::{Device, Geometry, IntoDevice},
    link::LinkBuilder,
};

use derive_more::Debug;

use super::Controller;
use crate::error::AUTDError;

#[derive(Builder, Debug)]
pub struct ControllerBuilder {
    #[debug(skip)]
    devices: Vec<Device>,
    #[get]
    #[set]
    fallback_parallel_threshold: usize,
    #[set]
    #[get]
    fallback_timeout: Duration,
    #[get]
    #[set]
    send_interval: Duration,
    #[get]
    #[set]
    receive_interval: Duration,
    #[cfg(target_os = "windows")]
    #[get]
    #[set]
    timer_resolution: Option<std::num::NonZeroU32>,
}

impl ControllerBuilder {
    #[must_use]
    pub(crate) fn new<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> ControllerBuilder {
        Self {
            devices: iter
                .into_iter()
                .enumerate()
                .map(|(i, d)| d.into_device(i as _))
                .collect(),
            fallback_parallel_threshold: 4,
            fallback_timeout: Duration::from_millis(20),
            send_interval: Duration::from_millis(1),
            receive_interval: Duration::from_millis(1),
            #[cfg(target_os = "windows")]
            timer_resolution: Some(std::num::NonZeroU32::MIN),
        }
    }

    pub async fn open<B: LinkBuilder>(
        self,
        link_builder: B,
    ) -> Result<Controller<B::L>, AUTDError> {
        self.open_with_timeout(link_builder, DEFAULT_TIMEOUT).await
    }

    #[tracing::instrument(level = "debug", skip(link_builder))]
    pub async fn open_with_timeout<B: LinkBuilder>(
        self,
        link_builder: B,
        timeout: Duration,
    ) -> Result<Controller<B::L>, AUTDError> {
        let geometry = Geometry::new(self.devices);
        Controller {
            link: link_builder.open(&geometry).await?,
            tx_buf: TxDatagram::new(geometry.num_devices()),
            rx_buf: vec![RxMessage::new(0, 0); geometry.num_devices()],
            geometry,
            fallback_parallel_threshold: self.fallback_parallel_threshold,
            fallback_timeout: self.fallback_timeout,
            send_interval: self.send_interval,
            receive_interval: self.receive_interval,
            #[cfg(target_os = "windows")]
            timer_resolution: self.timer_resolution,
        }
        .open_impl(timeout)
        .await
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{autd3_device::AUTD3, geometry::Vector3};

    use super::*;

    #[tokio::test]
    async fn geometry() -> anyhow::Result<()> {
        let autd =
            ControllerBuilder::new([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
                .open(crate::link::Nop::builder())
                .await?;

        assert_eq!(0, autd.geometry()[0].idx());
        autd.geometry()[0].iter().enumerate().for_each(|(i, tr)| {
            assert_eq!(i, tr.idx());
            assert_eq!(0, tr.dev_idx());
        });

        assert_eq!(1, autd.geometry()[1].idx());
        autd.geometry()[1].iter().enumerate().for_each(|(i, tr)| {
            assert_eq!(i, tr.idx());
            assert_eq!(1, tr.dev_idx());
        });

        Ok(())
    }
}
