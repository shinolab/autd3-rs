mod builder;
mod group;
mod timer;

use crate::{error::AUTDError, gain::Null, link::nop::Nop, prelude::Static};

use std::{fmt::Debug, hash::Hash, time::Duration};

use autd3_driver::{
    datagram::{Clear, Datagram, ForceFan, IntoDatagramWithTimeout, Silencer, Synchronize},
    derive::Builder,
    firmware::{
        cpu::{check_if_msg_is_processed, RxMessage, TxMessage},
        fpga::FPGAState,
        operation::{FirmwareVersionType, Operation, OperationHandler},
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry, IntoDevice},
    link::Link,
};

use timer::Timer;
use tracing;

pub use builder::ControllerBuilder;
pub use group::GroupGuard;
pub use timer::{AsyncSleeper, SpinSleeper, TimerStrategy};

use derive_more::{Deref, DerefMut};

#[derive(Builder, Deref, DerefMut)]
pub struct Controller<L: Link> {
    #[get(ref, ref_mut)]
    #[deref]
    #[deref_mut]
    link: L,
    #[get(ref, ref_mut)]
    geometry: Geometry,
    tx_buf: Vec<TxMessage>,
    rx_buf: Vec<RxMessage>,
    #[get]
    fallback_parallel_threshold: usize,
    #[get]
    fallback_timeout: Duration,
    #[get(ref)]
    timer: Timer,
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
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn send(&mut self, s: impl Datagram) -> Result<(), AUTDError> {
        let timeout = s.timeout();
        let parallel_threshold = s.parallel_threshold();

        let generator = s.operation_generator(&self.geometry)?;
        self.send_impl(
            OperationHandler::generate(generator, &self.geometry),
            timeout,
            parallel_threshold,
        )
        .await
    }

    pub(crate) async fn send_impl(
        &mut self,
        operations: Vec<(impl Operation, impl Operation)>,
        timeout: Option<Duration>,
        parallel_threshold: Option<usize>,
    ) -> Result<(), AUTDError> {
        let parallel_threshold = parallel_threshold.unwrap_or(self.fallback_parallel_threshold);
        let timeout = timeout.unwrap_or(self.fallback_timeout);
        let parallel = self.geometry.num_devices() > parallel_threshold;

        tracing::debug!("timeout: {:?}, parallel: {:?}", timeout, parallel);
        tracing::trace!("parallel_threshold: {:?}", parallel_threshold);

        self.link.update(&self.geometry).await?;

        self.timer
            .send(
                &self.geometry,
                &mut self.tx_buf,
                &mut self.rx_buf,
                &mut self.link,
                operations,
                timeout,
                parallel,
            )
            .await
    }

    pub(crate) async fn open_impl(mut self, timeout: Duration) -> Result<Self, AUTDError> {
        let timeout = Some(timeout);

        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data (here, we use `ForceFan` because it is the lightest).
        let _ = self
            .send(ForceFan::new(|_| false).with_timeout(timeout))
            .await;

        self.send((Clear::new(), Synchronize::new()).with_timeout(timeout))
            .await?;
        Ok(self)
    }

    async fn close_impl(&mut self) -> Result<(), AUTDError> {
        tracing::info!("Closing controller");

        if !self.link.is_open() {
            tracing::warn!("Link is already closed");
            return Ok(());
        }

        self.geometry.iter_mut().for_each(|dev| dev.enable = true);
        [
            self.send(Silencer::default().with_strict_mode(false)).await,
            self.send((Static::new(), Null::default())).await,
            self.send(Clear::new()).await,
            self.link.close().await.map_err(AUTDError::from),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn close(mut self) -> Result<(), AUTDError> {
        self.close_impl().await
    }

    async fn fetch_firminfo(&mut self, ty: FirmwareVersionType) -> Result<Vec<u8>, AUTDError> {
        self.send(ty).await.map_err(|e| {
            tracing::error!("Fetch firmware info failed: {:?}", e);
            AUTDError::ReadFirmwareVersionFailed(
                check_if_msg_is_processed(&self.tx_buf, &self.rx_buf).collect(),
            )
        })?;
        Ok(self.rx_buf.iter().map(|rx| rx.data()).collect())
    }

    pub async fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDError> {
        use FirmwareVersionType::*;
        let cpu_major = self.fetch_firminfo(CPUMajor).await?;
        let cpu_minor = self.fetch_firminfo(CPUMinor).await?;
        let fpga_major = self.fetch_firminfo(FPGAMajor).await?;
        let fpga_minor = self.fetch_firminfo(FPGAMinor).await?;
        let fpga_functions = self.fetch_firminfo(FPGAFunctions).await?;
        self.fetch_firminfo(Clear).await?;

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
        if !self.link.is_open() {
            return Err(AUTDError::Internal(
                autd3_driver::error::AUTDInternalError::LinkClosed,
            ));
        }
        if self.link.receive(&mut self.rx_buf).await? {
            Ok(self.rx_buf.iter().map(Option::<FPGAState>::from).collect())
        } else {
            Err(AUTDError::ReadFPGAStateFailed)
        }
    }
}

#[cfg(feature = "async-trait")]
impl<L: Link + 'static> Controller<L> {
    pub fn into_boxed_link(self) -> Controller<Box<dyn Link>> {
        let cnt = std::mem::ManuallyDrop::new(self);
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let tx_buf = unsafe { std::ptr::read(&cnt.tx_buf) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let fallback_parallel_threshold =
            unsafe { std::ptr::read(&cnt.fallback_parallel_threshold) };
        let fallback_timeout = unsafe { std::ptr::read(&cnt.fallback_timeout) };
        let timer = unsafe { std::ptr::read(&cnt.timer) };
        Controller {
            link: Box::new(link) as _,
            geometry,
            tx_buf,
            rx_buf,
            fallback_parallel_threshold,
            fallback_timeout,
            timer,
        }
    }

    pub fn from_boxed_link(cnt: Controller<Box<dyn Link>>) -> Controller<L> {
        let cnt = std::mem::ManuallyDrop::new(cnt);
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let tx_buf = unsafe { std::ptr::read(&cnt.tx_buf) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let fallback_parallel_threshold =
            unsafe { std::ptr::read(&cnt.fallback_parallel_threshold) };
        let fallback_timeout = unsafe { std::ptr::read(&cnt.fallback_timeout) };
        let timer = unsafe { std::ptr::read(&cnt.timer) };
        Controller {
            link: unsafe { *Box::from_raw(Box::into_raw(link) as *mut L) },
            geometry,
            tx_buf,
            rx_buf,
            fallback_parallel_threshold,
            fallback_timeout,
            timer,
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
                    let _ = self.close_impl().await;
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
        derive::{Gain, GainContext, GainContextGenerator, Segment},
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

    #[rstest::rstest]
    #[case(TimerStrategy::Std)]
    #[case(TimerStrategy::Spin(SpinSleeper::default()))]
    #[case(TimerStrategy::Async(AsyncSleeper::default()))]
    #[tokio::test(flavor = "multi_thread")]
    async fn open_with_timer(#[case] strategy: TimerStrategy) {
        assert!(Controller::builder([AUTD3::new(Vector3::zeros())])
            .with_timer_strategy(strategy)
            .open(Audit::builder())
            .await
            .is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn open_failed() {
        assert_eq!(
            Some(AUTDError::Internal(AUTDInternalError::SendDataFailed)),
            Controller::builder([AUTD3::new(Vector3::zeros())])
                .open(Audit::builder().with_down(true))
                .await
                .err()
        );
    }

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
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform::new(EmitIntensity::new(0x80))
                .init(&autd.geometry)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform::new(EmitIntensity::new(0x81))
                .init(&autd.geometry)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 1)
            );
            anyhow::Ok(())
        })?;

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
            autd.close_impl().await?;
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

    #[cfg(feature = "async-trait")]
    #[tokio::test(flavor = "multi_thread")]
    async fn into_boxed_link() -> anyhow::Result<()> {
        let autd = create_controller(1).await?;

        let mut autd = autd.into_boxed_link();

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

        let autd = Controller::<Audit>::from_boxed_link(autd);

        autd.geometry().iter().try_for_each(|dev| {
            assert_eq!(
                *Sine::new(150. * Hz).calc()?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform::new(EmitIntensity::new(0x80))
                .init(&autd.geometry)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform::new(EmitIntensity::new(0x81))
                .init(&autd.geometry)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 1)
            );
            anyhow::Ok(())
        })?;

        autd.close().await?;

        Ok(())
    }
}
