mod builder;
mod group;
/// Utilities for periodic operations.
pub mod timer;

use crate::{error::AUTDError, gain::Null, prelude::Static};

use std::time::Duration;

use autd3_driver::{
    datagram::{Clear, Datagram, ForceFan, IntoDatagramWithTimeout, Silencer, Synchronize},
    derive::Builder,
    error::AUTDDriverError,
    firmware::{
        cpu::{check_if_msg_is_processed, RxMessage, TxMessage},
        fpga::FPGAState,
        operation::{FirmwareVersionType, OperationHandler},
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry},
    link::Link,
};

use timer::Timer;
use tracing;

pub use builder::ControllerBuilder;
pub use group::Group;

use derive_more::{Deref, DerefMut};

/// A controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
#[derive(Builder, Deref, DerefMut)]
pub struct Controller<L: Link> {
    #[get(ref, ref_mut, no_doc)]
    link: L,
    #[get(ref, ref_mut, no_doc)]
    #[deref]
    #[deref_mut]
    geometry: Geometry,
    tx_buf: Vec<TxMessage>,
    rx_buf: Vec<RxMessage>,
    #[get(ref, no_doc)]
    timer: Timer,
}

impl<L: Link> Controller<L> {
    /// Sends a data to the devices.
    ///
    /// If the [`Datagram::timeout`] value is
    /// - greater than 0, this function waits until the sent data is processed by the device or the specified timeout time elapses. If it cannot be confirmed that the sent data has been processed by the device, [`AUTDDriverError::ConfirmResponseFailed`] is returned.
    /// - 0, the `send` function does not check whether the sent data has been processed by the device.
    ///
    /// The calculation of each [`Datagram`] is executed in parallel for each device if the number of enabled devices is greater than the [`Datagram::parallel_threshold`].
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn send(&mut self, s: impl Datagram) -> Result<(), AUTDDriverError> {
        let timeout = s.timeout();
        let parallel_threshold = s.parallel_threshold();
        self.link.trace(timeout, parallel_threshold);
        let generator = s.operation_generator(&self.geometry)?;
        self.timer.send(
            &self.geometry,
            &mut self.tx_buf,
            &mut self.rx_buf,
            &mut self.link,
            OperationHandler::generate(generator, &self.geometry),
            timeout,
            parallel_threshold,
        )
    }

    pub(crate) fn open_impl(mut self, timeout: Duration) -> Result<Self, AUTDError> {
        let timeout = Some(timeout);

        #[cfg(feature = "dynamic_freq")]
        {
            tracing::debug!(
                "Configuring ultrasound frequency to {:?}",
                autd3_driver::defined::ultrasound_freq()
            );
            self.send(autd3_driver::datagram::ConfigureFPGAClock::new().with_timeout(timeout))?;
        }

        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data (here, we use `ForceFan` because it is the lightest).
        let _ = self.send(ForceFan::new(|_| false).with_timeout(timeout));

        self.send((Clear::new(), Synchronize::new()).with_timeout(timeout))?;
        Ok(self)
    }

    fn close_impl(&mut self) -> Result<(), AUTDDriverError> {
        tracing::info!("Closing controller");

        if !self.link.is_open() {
            tracing::warn!("Link is already closed");
            return Ok(());
        }

        self.geometry.iter_mut().for_each(|dev| dev.enable = true);
        [
            self.send(Silencer::default().with_strict_mode(false)),
            self.send((Static::new(), Null::default())),
            self.send(Clear::new()),
            self.link.close(),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }

    /// Closes the controller.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn close(mut self) -> Result<(), AUTDDriverError> {
        self.close_impl()
    }

    fn fetch_firminfo(&mut self, ty: FirmwareVersionType) -> Result<Vec<u8>, AUTDError> {
        self.send(ty).map_err(|e| {
            tracing::error!("Fetch firmware info failed: {:?}", e);
            AUTDError::ReadFirmwareVersionFailed(
                check_if_msg_is_processed(&self.tx_buf, &self.rx_buf).collect(),
            )
        })?;
        Ok(self.rx_buf.iter().map(|rx| rx.data()).collect())
    }

    /// Returns  the firmware version of the devices.
    pub fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDError> {
        use autd3_driver::firmware::version::{CPUVersion, FPGAVersion, Major, Minor};
        use FirmwareVersionType::*;

        let cpu_major = self.fetch_firminfo(CPUMajor)?;
        let cpu_minor = self.fetch_firminfo(CPUMinor)?;
        let fpga_major = self.fetch_firminfo(FPGAMajor)?;
        let fpga_minor = self.fetch_firminfo(FPGAMinor)?;
        let fpga_functions = self.fetch_firminfo(FPGAFunctions)?;
        self.fetch_firminfo(Clear)?;

        Ok(self
            .geometry
            .devices()
            .map(|dev| {
                FirmwareVersion::new(
                    dev.idx(),
                    CPUVersion::new(Major(cpu_major[dev.idx()]), Minor(cpu_minor[dev.idx()])),
                    FPGAVersion::new(
                        Major(fpga_major[dev.idx()]),
                        Minor(fpga_minor[dev.idx()]),
                        fpga_functions[dev.idx()],
                    ),
                )
            })
            .collect())
    }

    /// Returns the FPGA state of the devices.
    ///
    /// To get the state of devices, enable reads FPGA state mode by [`ReadsFPGAState`] before calling this method.
    /// The returned value is [`None`] if the reads FPGA state mode is disabled for the device.
    ///
    /// # Examples
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # fn main() -> Result<(), AUTDError> {
    /// let mut autd = Controller::builder([AUTD3::new(Point3::origin())]).open(Nop::builder())?;
    ///
    /// autd.send(ReadsFPGAState::new(|_| true))?;
    ///
    /// let states = autd.fpga_state()?;
    /// Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadsFPGAState`]: autd3_driver::datagram::ReadsFPGAState
    pub fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDError> {
        if !self.link.is_open() {
            return Err(AUTDError::Driver(
                autd3_driver::error::AUTDDriverError::LinkClosed,
            ));
        }
        if self.link.receive(&mut self.rx_buf)? {
            Ok(self.rx_buf.iter().map(Option::from).collect())
        } else {
            Err(AUTDError::ReadFPGAStateFailed)
        }
    }
}

impl<'a, L: Link> IntoIterator for &'a Controller<L> {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter()
    }
}

impl<'a, L: Link> IntoIterator for &'a mut Controller<L> {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter_mut()
    }
}

impl<L: Link + 'static> Controller<L> {
    /// Converts `Controller<L>` into a `Controller<Box<dyn Link>>`.
    pub fn into_boxed_link(self) -> Controller<Box<dyn Link>> {
        let cnt = std::mem::ManuallyDrop::new(self);
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let tx_buf = unsafe { std::ptr::read(&cnt.tx_buf) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let timer = unsafe { std::ptr::read(&cnt.timer) };
        Controller {
            link: Box::new(link) as _,
            geometry,
            tx_buf,
            rx_buf,
            timer,
        }
    }

    /// Converts `Controller<Box<dyn Link>>` into a `Controller<L>`.
    ///
    /// # Safety
    ///
    /// This function must be used only when converting an instance created by [`Controller::into_boxed_link`] back to the original [`Controller<L>`].
    ///
    pub unsafe fn from_boxed_link(cnt: Controller<Box<dyn Link>>) -> Controller<L> {
        let cnt = std::mem::ManuallyDrop::new(cnt);
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let tx_buf = unsafe { std::ptr::read(&cnt.tx_buf) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let timer = unsafe { std::ptr::read(&cnt.timer) };
        Controller {
            link: unsafe { *Box::from_raw(Box::into_raw(link) as *mut L) },
            geometry,
            tx_buf,
            rx_buf,
            timer,
        }
    }
}

impl<L: Link> Drop for Controller<L> {
    fn drop(&mut self) {
        if !self.link.is_open() {
            return;
        }
        let _ = self.close_impl();
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        autd3_device::AUTD3,
        defined::Hz,
        derive::{Gain, GainContext, GainContextGenerator, Segment},
        geometry::Point3,
    };

    use crate::{controller::timer::*, link::Audit, prelude::*};

    use super::*;

    // GRCOV_EXCL_START
    pub fn create_controller(dev_num: usize) -> anyhow::Result<Controller<Audit>> {
        Ok(
            Controller::builder((0..dev_num).map(|_| AUTD3::new(Point3::origin())))
                .open(Audit::builder())?,
        )
    }
    // GRCOV_EXCL_STOP

    #[rstest::rstest]
    #[case(TimerStrategy::Std(StdSleeper::default()))]
    #[case(TimerStrategy::Spin(SpinSleeper::default()))]
    #[cfg_attr(target_os = "windows", case(TimerStrategy::Waitable(WaitableSleeper::new().unwrap())))]
    #[test]
    fn open_with_timer(#[case] strategy: TimerStrategy) {
        assert!(Controller::builder([AUTD3::new(Point3::origin())])
            .with_timer_strategy(strategy)
            .open(Audit::builder())
            .is_ok());
    }

    #[test]
    fn open_failed() {
        assert_eq!(
            Some(AUTDError::Driver(AUTDDriverError::SendDataFailed)),
            Controller::builder([AUTD3::new(Point3::origin())])
                .open(Audit::builder().with_down(true))
                .err()
        );
    }

    #[test]
    fn send() -> anyhow::Result<()> {
        let mut autd = create_controller(1)?;
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
        ))?;

        autd.iter().try_for_each(|dev| {
            assert_eq!(
                *Sine::new(150. * Hz).calc()?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform::new(EmitIntensity::new(0x80))
                .init(&autd.geometry, None)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform::new(EmitIntensity::new(0x81))
                .init(&autd.geometry, None)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 1)
            );
            anyhow::Ok(())
        })?;

        autd.close()?;

        Ok(())
    }

    #[test]
    fn firmware_version() -> anyhow::Result<()> {
        use autd3_driver::firmware::version::{CPUVersion, FPGAVersion};

        let mut autd = create_controller(1)?;
        assert_eq!(
            vec![FirmwareVersion::new(
                0,
                CPUVersion::new(
                    FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
                    FirmwareVersion::LATEST_VERSION_NUM_MINOR
                ),
                FPGAVersion::new(
                    FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
                    FirmwareVersion::LATEST_VERSION_NUM_MINOR,
                    FPGAVersion::ENABLED_EMULATOR_BIT
                )
            )],
            autd.firmware_version()?
        );
        Ok(())
    }

    #[test]
    fn firmware_version_err() -> anyhow::Result<()> {
        let mut autd = create_controller(2)?;
        autd.link_mut().break_down();
        assert_eq!(
            Err(AUTDError::ReadFirmwareVersionFailed(vec![false, false])),
            autd.firmware_version()
        );
        Ok(())
    }

    #[test]
    fn close() -> anyhow::Result<()> {
        {
            let mut autd = create_controller(1)?;
            autd.close_impl()?;
            autd.close()?;
        }

        {
            let mut autd = create_controller(1)?;
            autd.link_mut().break_down();
            assert_eq!(
                Err(AUTDDriverError::LinkError("broken".to_owned())),
                autd.close()
            );
        }

        {
            let mut autd = create_controller(1)?;
            autd.link_mut().down();
            assert_eq!(Err(AUTDDriverError::SendDataFailed), autd.close());
        }

        Ok(())
    }

    #[test]
    fn fpga_state() -> anyhow::Result<()> {
        let mut autd =
            Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
                .open(Audit::builder())?;

        autd.send(ReadsFPGAState::new(|_| true))?;
        {
            autd.link_mut()[0].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state()?;
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

            let states = autd.fpga_state()?;
            assert_eq!(2, states.len());
            assert!(!states[0]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
            assert!(states[1]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
        }

        autd.send(ReadsFPGAState::new(|dev| dev.idx() == 1))?;
        {
            let states = autd.fpga_state()?;
            assert_eq!(2, states.len());
            assert!(states[0].is_none());
            assert!(states[1]
                .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                .is_thermal_assert());
        }

        Ok(())
    }

    #[test]
    fn into_iter() -> anyhow::Result<()> {
        let mut autd = create_controller(1)?;

        for dev in &mut autd {
            dev.sound_speed = 300e3 * mm;
        }

        for dev in &autd {
            assert_eq!(300e3 * mm, dev.sound_speed);
        }

        Ok(())
    }

    #[test]
    fn into_boxed_link() -> anyhow::Result<()> {
        let autd = create_controller(1)?;

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
        ))?;

        let autd = unsafe { Controller::<Audit>::from_boxed_link(autd) };

        autd.iter().try_for_each(|dev| {
            assert_eq!(
                *Sine::new(150. * Hz).calc()?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform::new(EmitIntensity::new(0x80))
                .init(&autd.geometry, None)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform::new(EmitIntensity::new(0x81))
                .init(&autd.geometry, None)?
                .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 1)
            );
            anyhow::Ok(())
        })?;

        autd.close()?;

        Ok(())
    }
}
