mod builder;
mod group;

use std::{fmt::Debug, hash::Hash, time::Duration};

use autd3_driver::{
    datagram::{Clear, Datagram, IntoDatagramWithTimeout, Synchronize},
    defined::DEFAULT_TIMEOUT,
    derive::{tracing, Builder},
    firmware::{
        cpu::{RxMessage, TxDatagram},
        fpga::FPGAState,
        operation::{Operation, OperationHandler},
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry, IntoDevice},
    link::{send_receive, Link},
};

use crate::{
    error::{AUTDError, ReadFirmwareVersionState},
    gain::Null,
    link::nop::Nop,
    prelude::Static,
};

pub use builder::ControllerBuilder;
pub use group::GroupGuard;

#[derive(Builder)]
pub struct Controller<L: Link> {
    #[get]
    link: L,
    #[get]
    geometry: Geometry,
    tx_buf: TxDatagram,
    rx_buf: Vec<RxMessage>,
    #[get]
    parallel_threshold: usize,
    #[get]
    last_parallel_threshold: usize,
    #[get]
    send_interval: Duration,
    #[cfg(target_os = "windows")]
    #[get]
    timer_resolution: std::num::NonZeroU32,
}

impl Controller<Nop> {
    pub fn builder<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> ControllerBuilder {
        ControllerBuilder::new(iter)
    }
}

impl<L: Link> Controller<L> {
    #[must_use]
    pub fn group<K: Hash + Eq + Clone + Debug, F: Fn(&Device) -> Option<K>>(
        &mut self,
        f: F,
    ) -> GroupGuard<K, L, F> {
        GroupGuard::new(self, f)
    }

    #[must_use]
    pub fn link_mut(&mut self) -> &mut L {
        &mut self.link
    }
}

impl<L: Link> Controller<L> {
    #[tracing::instrument(skip(self, s))]
    pub async fn send(&mut self, s: impl Datagram) -> Result<(), AUTDError> {
        let timeout = s.timeout();
        let parallel_threshold = s.parallel_threshold().unwrap_or(self.parallel_threshold);

        s.trace(&self.geometry);

        let generator = s.operation_generator(&self.geometry)?;
        let mut operations = OperationHandler::generate(generator, &self.geometry);
        self.send_impl(&mut operations, timeout, parallel_threshold)
            .await
    }

    #[tracing::instrument(skip(self, operations))]
    pub(crate) async fn send_impl(
        &mut self,
        operations: &mut [(impl Operation, impl Operation)],
        timeout: Option<Duration>,
        parallel_threshold: usize,
    ) -> Result<(), AUTDError> {
        self.last_parallel_threshold = parallel_threshold;
        self.link.update(&self.geometry).await?;
        loop {
            OperationHandler::pack(
                operations,
                &self.geometry,
                &mut self.tx_buf,
                parallel_threshold,
            )?;
            let start = tokio::time::Instant::now();
            send_receive(&mut self.link, &self.tx_buf, &mut self.rx_buf, timeout).await?;
            if OperationHandler::is_done(operations) {
                return Ok(());
            }
            tokio::time::sleep_until(start + self.send_interval).await;
        }
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn open_impl(&mut self, timeout: Duration) -> Result<(), AUTDError> {
        #[cfg(target_os = "windows")]
        unsafe {
            tracing::debug!("Set timer resulution: {:?}", self.timer_resolution);
            windows::Win32::Media::timeBeginPeriod(self.timer_resolution.get());
        }

        self.send((Clear::new(), Synchronize::new()).with_timeout(timeout))
            .await?; // GRCOV_EXCL_LINE
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), AUTDError> {
        if !self.link.is_open() {
            return Ok(());
        }
        self.geometry.iter_mut().for_each(|dev| dev.enable = true);
        self.send(
            autd3_driver::datagram::SilencerFixedCompletionSteps::default().with_strict_mode(false),
        )
        .await?;
        self.send((Static::new(), Null::default())).await?;
        self.send(Clear::new()).await?;
        self.link.close().await?;
        Ok(())
    }

    pub async fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDError> {
        let mut operations = self
            .geometry
            .iter()
            .map(|_| {
                (
                    autd3_driver::firmware::operation::FirmInfoOp::default(),
                    autd3_driver::firmware::operation::NullOp::default(),
                )
            })
            .collect::<Vec<_>>();

        macro_rules! pack_and_send {
            ($operations:expr, $link:expr, $geometry:expr, $tx_buf:expr, $rx_buf:expr) => {
                OperationHandler::pack($operations, $geometry, $tx_buf, usize::MAX)?;
                if autd3_driver::link::send_receive($link, $tx_buf, $rx_buf, Some(DEFAULT_TIMEOUT))
                    .await
                    .is_err()
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
            &mut operations,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf, // GRCOV_EXCL_LINE
            &mut self.rx_buf  // GRCOV_EXCL_LINE
        );
        let cpu_versions = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut operations,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf, // GRCOV_EXCL_LINE
            &mut self.rx_buf  // GRCOV_EXCL_LINE
        );
        let cpu_versions_minor = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut operations,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf, // GRCOV_EXCL_LINE
            &mut self.rx_buf  // GRCOV_EXCL_LINE
        );
        let fpga_versions = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut operations,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf, // GRCOV_EXCL_LINE
            &mut self.rx_buf  // GRCOV_EXCL_LINE
        );
        let fpga_versions_minor = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut operations,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf, // GRCOV_EXCL_LINE
            &mut self.rx_buf  // GRCOV_EXCL_LINE
        );
        let fpga_functions = self.rx_buf.iter().map(|rx| rx.data()).collect::<Vec<_>>();

        pack_and_send!(
            &mut operations,
            &mut self.link,
            &self.geometry,
            &mut self.tx_buf, // GRCOV_EXCL_LINE
            &mut self.rx_buf  // GRCOV_EXCL_LINE
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

    pub async fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDError> {
        if self.link.receive(&mut self.rx_buf).await? {
            Ok(self.rx_buf.iter().map(Option::<FPGAState>::from).collect())
        } else {
            Err(AUTDError::ReadFPGAStateFailed)
        }
    }
}

#[cfg_attr(feature = "capi", allow(clippy::needless_return))]
impl<L: Link> Drop for Controller<L> {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            windows::Win32::Media::timeEndPeriod(self.timer_resolution.get());
        }
        if !self.link.is_open() {
            return;
        }
        #[cfg(not(feature = "capi"))]
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
    use autd3_driver::{
        autd3_device::AUTD3,
        defined::Hz,
        derive::{Gain, Segment},
        geometry::Vector3,
    };

    use crate::{link::Audit, prelude::*};

    use super::*;

    // GRCOV_EXCL_START
    pub async fn create_controller(dev_num: usize) -> anyhow::Result<Controller<Audit>> {
        Ok(
            Controller::builder((0..dev_num).map(|_| AUTD3::new(Vector3::zeros())))
                .open(Audit::builder())
                .await?,
        )
    }
    // GRCOV_EXCL_STOP

    #[tokio::test(flavor = "multi_thread")]
    async fn send() -> anyhow::Result<()> {
        let mut autd = create_controller(1).await?;
        autd.send((
            Sine::new(150. * Hz),
            GainSTM::new(
                1. * Hz,
                [
                    Uniform::new(EmitIntensity::new(0x80)),
                    Uniform::new(EmitIntensity::new(0x81)),
                ]
                .into_iter(),
            )?,
        ))
        .await?;

        autd.geometry().iter().try_for_each(|dev| {
            assert_eq!(
                Sine::new(150. * Hz).calc()?,
                autd.link[dev.idx()].fpga().modulation(Segment::S0)
            );
            let f = Uniform::new(EmitIntensity::new(0x80)).calc(&autd.geometry)?(dev);
            assert_eq!(
                dev.iter().map(f).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives(Segment::S0, 0)
            );
            let f = Uniform::new(EmitIntensity::new(0x81)).calc(&autd.geometry)?(dev);
            assert_eq!(
                dev.iter().map(f).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives(Segment::S0, 1)
            );
            anyhow::Ok(())
        })?;

        autd.close().await?;
        autd.close().await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn firmware_version() -> anyhow::Result<()> {
        let mut autd = create_controller(1).await?;
        assert_eq!(
            vec![FirmwareVersion::new(
                0,
                FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
                FirmwareVersion::LATEST_VERSION_NUM_MINOR,
                FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
                FirmwareVersion::LATEST_VERSION_NUM_MINOR,
                FirmwareVersion::ENABLED_EMULATOR_BIT
            )],
            autd.firmware_version().await?
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn close() -> anyhow::Result<()> {
        {
            let mut autd = create_controller(1).await?;
            autd.close().await?;
            autd.close().await?;
        }

        {
            let mut autd = create_controller(1).await?;
            autd.link_mut().break_down();
            assert_eq!(
                Err(AUTDError::Internal(AUTDInternalError::LinkError(
                    "broken".to_owned()
                ))),
                autd.close().await
            );
        }

        {
            let mut autd = create_controller(1).await?;
            autd.link_mut().down();
            assert_eq!(
                Err(AUTDError::Internal(AUTDInternalError::SendDataFailed)),
                autd.close().await
            );
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fpga_state() -> anyhow::Result<()> {
        let mut autd =
            Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
                .open(Audit::builder())
                .await?;

        autd.send(ReadsFPGAState::new(|_| true)).await?;
        {
            autd.link_mut()[0].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state().await?;
            assert_eq!(2, states.len());
            assert!(states[0]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
            assert!(!states[1]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
        }

        {
            autd.link_mut()[0].fpga_mut().deassert_thermal_sensor();
            autd.link_mut()[1].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state().await?;
            assert_eq!(2, states.len());
            assert!(!states[0]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
            assert!(states[1]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
        }

        autd.send(ReadsFPGAState::new(|dev| dev.idx() == 1)).await?;
        {
            let states = autd.fpga_state().await?;
            assert_eq!(2, states.len());
            assert!(states[0].is_none());
            assert!(states[1]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn last_parallel_threshold() -> anyhow::Result<()> {
        let mut autd =
            Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
                .with_parallel_threshold(0)
                .open(Audit::builder())
                .await?;
        assert_eq!(usize::MAX, autd.last_parallel_threshold);

        autd.send(Null::new()).await?;
        assert_eq!(0, autd.last_parallel_threshold);

        autd.send(Static::new()).await?;
        assert_eq!(usize::MAX, autd.last_parallel_threshold);

        autd.send(Static::new().with_parallel_threshold(10)).await?;
        assert_eq!(10, autd.last_parallel_threshold);

        autd.send((Static::new(), Static::new()).with_parallel_threshold(5))
            .await?;
        assert_eq!(5, autd.last_parallel_threshold);

        Ok(())
    }
}
