use std::time::Duration;

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxMessage},
    geometry::{Device, Geometry, IntoDevice},
    link::LinkBuilder,
};

use derive_more::Debug;
use spin_sleep::SpinSleeper;
use zerocopy::FromZeros;

use super::{
    timer::{Timer, TimerStrategy},
    Controller,
};
use crate::{error::AUTDError, link::Nop};

/// A builder for creating a [`Controller`] instance.
#[derive(Builder, Debug)]
pub struct ControllerBuilder {
    #[debug(skip)]
    #[get(take)]
    /// Takes the devices out of the builder.
    devices: Vec<Device>,
    #[get]
    #[set]
    /// The default parallel threshold when no threshold is specified for the [`Datagram`](crate::driver::datagram::Datagram) to be sent. The default value is 4.
    default_parallel_threshold: usize,
    #[set]
    #[get]
    /// The default timeout when no timeout is specified for the [`Datagram`](crate::driver::datagram::Datagram) to be sent. The default value is 20ms.
    default_timeout: Duration,
    #[get]
    #[set]
    /// The duration between sending operations. The default value is 1ms.
    send_interval: Duration,
    #[get]
    #[set]
    /// The duration between receiving operations. The default value is 1ms.
    receive_interval: Duration,
    #[get(ref)]
    #[set]
    /// The strategy used for timing operations. The default value is [`TimerStrategy::Spin`](crate::controller::timer::TimerStrategy::Spin) with the default [`SpinSleeper`](spin_sleep::SpinSleeper).
    timer_strategy: TimerStrategy,
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
            default_parallel_threshold: 4,
            default_timeout: Duration::from_millis(20),
            send_interval: Duration::from_millis(1),
            receive_interval: Duration::from_millis(1),
            timer_strategy: TimerStrategy::Spin(SpinSleeper::default()),
        }
    }

    /// Equivalent to [`open_with_timeout`] with a timeout of [`DEFAULT_TIMEOUT`].
    ///
    /// [`open_with_timeout`]: ControllerBuilder::open_with_timeout
    /// [`DEFAULT_TIMEOUT`]: autd3_driver::defined::DEFAULT_TIMEOUT
    pub async fn open<B: LinkBuilder>(
        self,
        link_builder: B,
    ) -> Result<Controller<B::L>, AUTDError> {
        self.open_with_timeout(link_builder, DEFAULT_TIMEOUT).await
    }

    /// Opens a controller with a timeout.
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub async fn open_with_timeout<B: LinkBuilder>(
        self,
        link_builder: B,
        timeout: Duration,
    ) -> Result<Controller<B::L>, AUTDError> {
        tracing::debug!("Opening a controller: {:?} (timeout = {:?})", self, timeout);
        let geometry = Geometry::new(self.devices, self.default_parallel_threshold);
        Controller {
            link: link_builder.open(&geometry).await?,
            tx_buf: vec![TxMessage::new_zeroed(); geometry.len()], // Do not use `num_devices` here because the devices may be disabled.
            rx_buf: vec![RxMessage::new(0, 0); geometry.len()],
            geometry,
            timer: Timer {
                send_interval: self.send_interval,
                receive_interval: self.receive_interval,
                strategy: self.timer_strategy,
                default_timeout: self.default_timeout,
            },
        }
        .open_impl(timeout)
        .await
    }
}

impl Controller<Nop> {
    /// Creates a new [`ControllerBuilder`] with the given devices.
    #[must_use]
    pub fn builder<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> ControllerBuilder {
        ControllerBuilder::new(iter)
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

        assert_eq!(0, autd[0].idx());
        autd[0].iter().enumerate().for_each(|(i, tr)| {
            assert_eq!(i, tr.idx());
            assert_eq!(0, tr.dev_idx());
        });

        assert_eq!(1, autd[1].idx());
        autd[1].iter().enumerate().for_each(|(i, tr)| {
            assert_eq!(i, tr.idx());
            assert_eq!(1, tr.dev_idx());
        });

        Ok(())
    }
}
