mod builder;
mod group;

use std::{fmt::Debug, hash::Hash, time::Duration};

use autd3_driver::{
    datagram::{Clear, Datagram, FetchFirmInfo, IntoDatagramWithTimeout, Synchronize},
    derive::{tracing, Builder, Itertools},
    firmware::{
        cpu::{check_if_msg_is_processed, RxMessage, TxDatagram},
        fpga::FPGAState,
        operation::{Operation, OperationHandler},
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry, IntoDevice},
    link::{send_receive, Link},
};

use crate::{error::AUTDError, gain::Null, link::nop::Nop, prelude::Static};

pub use builder::ControllerBuilder;
pub use group::GroupGuard;

#[derive(Builder)]
pub struct Controller<L: Link> {
    #[get(ref, ref_mut)]
    link: L,
    #[get(ref, ref_mut)]
    geometry: Geometry,
    tx_buf: TxDatagram,
    rx_buf: Vec<RxMessage>,
    #[get]
    parallel_threshold: usize,
    #[get]
    send_interval: Duration,
    #[cfg(target_os = "windows")]
    #[get]
    timer_resolution: std::num::NonZeroU32,
}

impl Controller<Nop> {
    #[must_use]
    pub fn builder<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> ControllerBuilder {
        ControllerBuilder::new(iter)
    }
}

impl<L: Link> Controller<L> {
    #[must_use]
    pub fn group<K: Hash + Eq + Clone + Debug, F: Fn(&Device) -> Option<K>>(
        &mut self,
        f: F,
    ) -> GroupGuard<K, L> {
        GroupGuard::new(self, f)
    }
}

impl<L: Link> Controller<L> {
    #[tracing::instrument(skip(self, s))]
    pub async fn send(&mut self, s: impl Datagram) -> Result<(), AUTDError> {
        let timeout = s.timeout();
        let parallel_threshold = s.parallel_threshold();

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
        parallel_threshold: Option<usize>,
    ) -> Result<(), AUTDError> {
        let parallel_threshold = parallel_threshold.unwrap_or(self.parallel_threshold);
        let timeout = timeout.unwrap_or(self.link.timeout());

        self.link.update(&self.geometry).await?;
        loop {
            OperationHandler::pack(
                operations,
                &self.geometry,
                &mut self.tx_buf,
                parallel_threshold,
            )?;

            self.link
                .trace(&self.tx_buf, &mut self.rx_buf, timeout, parallel_threshold);

            // GRCOV_EXCL_START
            tracing::trace!(
                "send: {}",
                self.tx_buf.iter().format_with(", ", |elt, f| {
                    f(&format_args!("({:?}, {:#04X})", elt.header, elt.payload[0]))
                })
            );
            // GRCOV_EXCL_STOP

            let start = tokio::time::Instant::now();
            send_receive(&mut self.link, &self.tx_buf, &mut self.rx_buf, timeout).await?;
            if OperationHandler::is_done(operations) {
                return Ok(());
            }
            tokio::time::sleep_until(start + self.send_interval).await;
        }
    }

    #[must_use]
    #[tracing::instrument(skip(self))]
    pub(crate) async fn open_impl(mut self, timeout: Duration) -> Result<Self, AUTDError> {
        #[cfg(target_os = "windows")]
        unsafe {
            tracing::debug!("Set timer resolution: {:?}", self.timer_resolution);
            windows::Win32::Media::timeBeginPeriod(self.timer_resolution.get());
        }
        self.send((Clear::new(), Synchronize::new()).with_timeout(timeout))
            .await?; // GRCOV_EXCL_LINE
        Ok(self)
    }

    pub async fn close(&mut self) -> Result<(), AUTDError> {
        if !self.link.is_open() {
            return Ok(());
        }
        self.geometry.iter_mut().for_each(|dev| dev.enable = true);
        self.send(autd3_driver::datagram::Silencer::<autd3_driver::datagram::FixedCompletionTime>::default().with_strict_mode(false))
            .await?;
        self.send((Static::new(), Null::default())).await?;
        self.send(Clear::new()).await?;
        self.link.close().await?;
        Ok(())
    }

    async fn fetch_firminfo(&mut self, ty: FetchFirmInfo) -> Result<Vec<u8>, AUTDError> {
        self.send(ty).await.map_err(|_| {
            AUTDError::ReadFirmwareVersionFailed(
                check_if_msg_is_processed(&self.tx_buf, &mut self.rx_buf).collect(),
            )
        })?;
        Ok(self.rx_buf.iter().map(|rx| rx.data()).collect())
    }

    pub async fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDError> {
        let cpu_major = self.fetch_firminfo(FetchFirmInfo::CPUMajor).await?;
        let cpu_minor = self.fetch_firminfo(FetchFirmInfo::CPUMinor).await?;
        let fpga_major = self.fetch_firminfo(FetchFirmInfo::FPGAMajor).await?;
        let fpga_minor = self.fetch_firminfo(FetchFirmInfo::FPGAMinor).await?;
        let fpga_functions = self.fetch_firminfo(FetchFirmInfo::FPGAFunctions).await?;
        self.fetch_firminfo(FetchFirmInfo::Clear).await?;

        Ok(self
            .geometry
            .devices()
            .map(|dev| {
                FirmwareVersion::new(
                    dev.idx(),
                    cpu_major[dev.idx()],
                    cpu_minor[dev.idx()],
                    fpga_major[dev.idx()],
                    fpga_minor[dev.idx()],
                    fpga_functions[dev.idx()],
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
                *Sine::new(150. * Hz).calc()?,
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
    async fn firmware_version_err() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;
        autd.link_mut().break_down();
        assert_eq!(
            Err(AUTDError::ReadFirmwareVersionFailed(vec![false, false])),
            autd.firmware_version().await
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
}
