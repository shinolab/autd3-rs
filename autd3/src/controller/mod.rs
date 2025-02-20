mod group;
mod sender;

use crate::{error::AUTDError, gain::Null, modulation::Static};

use autd3_core::{defined::DEFAULT_TIMEOUT, geometry::IntoDevice, link::Link};
use autd3_driver::{
    datagram::{Clear, Datagram, FixedCompletionSteps, ForceFan, Silencer, Synchronize},
    error::AUTDDriverError,
    firmware::{
        cpu::{check_if_msg_is_processed, RxMessage, TxMessage},
        fpga::FPGAState,
        operation::{FirmwareVersionType, Operation, OperationGenerator},
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry},
};

#[cfg(target_os = "windows")]
pub use sender::WaitableSleeper;
pub use sender::{
    sleep::Sleep, ParallelMode, Sender, SenderOption, SpinSleeper, SpinStrategy, StdSleeper,
};

use derive_more::{Deref, DerefMut};
use getset::{Getters, MutGetters};
use tracing;
use zerocopy::FromZeros;

/// A controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
#[derive(Deref, DerefMut, Getters, MutGetters)]
pub struct Controller<L: Link> {
    /// The link to the devices.
    #[getset(get = "pub", get_mut = "pub")]
    link: L,
    /// The geometry of the devices.
    #[getset(get = "pub", get_mut = "pub")]
    #[deref]
    #[deref_mut]
    geometry: Geometry,
    tx_buf: Vec<TxMessage>,
    rx_buf: Vec<RxMessage>,
}

impl<L: Link> Controller<L> {
    /// Equivalent to [`Self::open_with_option`] with a timeout of [`DEFAULT_TIMEOUT`].
    pub fn open<D: IntoDevice, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDError> {
        Self::open_with_option::<D, F, SpinSleeper>(
            devices,
            link,
            SenderOption {
                timeout: Some(DEFAULT_TIMEOUT),
                ..Default::default()
            },
        )
    }

    /// Opens a controller with a [`SenderOption`].
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub fn open_with_option<D: IntoDevice, F: IntoIterator<Item = D>, S: Sleep>(
        devices: F,
        mut link: L,
        option: SenderOption<S>,
    ) -> Result<Self, AUTDError> {
        tracing::debug!("Opening a controller with option {:?})", option);

        let devices = devices
            .into_iter()
            .enumerate()
            .map(|(i, d)| d.into_device(i as _))
            .collect();

        let geometry = Geometry::new(devices);
        link.open(&geometry)?;
        Controller {
            link,
            tx_buf: vec![TxMessage::new_zeroed(); geometry.len()], // Do not use `num_devices` here because the devices may be disabled.
            rx_buf: vec![RxMessage::new(0, 0); geometry.len()],
            geometry,
        }
        .open_impl(option)
    }

    /// Returns the [`Sender`] to send data to the devices.
    pub fn sender<S: Sleep>(&mut self, option: SenderOption<S>) -> Sender<'_, L, S> {
        Sender {
            link: &mut self.link,
            geometry: &mut self.geometry,
            tx: &mut self.tx_buf,
            rx: &mut self.rx_buf,
            option,
        }
    }

    /// Sends a data to the devices. This is a shortcut for [`Sender::send`].
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        self.sender(SenderOption::<SpinSleeper>::default()).send(s)
    }

    pub(crate) fn open_impl<S: Sleep>(
        mut self,
        option: SenderOption<S>,
    ) -> Result<Self, AUTDError> {
        let mut sender = self.sender(option);

        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data (here, we use `ForceFan` because it is the lightest).
        let _ = sender.send(ForceFan::new(|_| false));

        #[cfg(feature = "dynamic_freq")]
        {
            tracing::debug!(
                "Configuring ultrasound frequency to {:?}",
                autd3_driver::defined::ultrasound_freq()
            );
            sender.send(autd3_driver::datagram::ConfigureFPGAClock::new())?;
        }

        sender.send((Clear::new(), Synchronize::new()))?;
        Ok(self)
    }

    fn close_impl<S: Sleep>(&mut self, option: SenderOption<S>) -> Result<(), AUTDDriverError> {
        tracing::info!("Closing controller");

        if !self.link.is_open() {
            tracing::warn!("Link is already closed");
            return Ok(());
        }

        self.geometry.iter_mut().for_each(|dev| dev.enable = true);

        let mut sender = self.sender(option);

        [
            sender.send(Silencer {
                config: FixedCompletionSteps {
                    strict_mode: false,
                    ..Default::default()
                },
                target: autd3_driver::firmware::fpga::SilencerTarget::Intensity,
            }),
            sender.send((Static::default(), Null)),
            sender.send(Clear {}),
            Ok(self.link.close()?),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }

    /// Closes the controller.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn close(mut self) -> Result<(), AUTDDriverError> {
        self.close_impl(SenderOption::<SpinSleeper>::default())
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
            .map(|dev| FirmwareVersion {
                idx: dev.idx(),
                cpu: CPUVersion {
                    major: Major(cpu_major[dev.idx()]),
                    minor: Minor(cpu_minor[dev.idx()]),
                },
                fpga: FPGAVersion {
                    major: Major(fpga_major[dev.idx()]),
                    minor: Minor(fpga_minor[dev.idx()]),
                    function_bits: fpga_functions[dev.idx()],
                },
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
    /// let mut autd = Controller::open([AUTD3::default()], Nop::new())?;
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
        self.link.receive(&mut self.rx_buf)?;
        Ok(self.rx_buf.iter().map(FPGAState::from_rx).collect())
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
        Controller {
            link: Box::new(link) as _,
            geometry,
            tx_buf,
            rx_buf,
        }
    }

    /// Converts `Controller<Box<dyn Link>>` into a `Controller<L>`.
    ///
    /// # Safety
    ///
    /// This function must be used only when converting an instance created by [`Controller::into_boxed_link`] back to the original [`Controller<L>`].
    pub unsafe fn from_boxed_link(cnt: Controller<Box<dyn Link>>) -> Controller<L> {
        let cnt = std::mem::ManuallyDrop::new(cnt);
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let tx_buf = unsafe { std::ptr::read(&cnt.tx_buf) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        Controller {
            link: unsafe { *Box::from_raw(Box::into_raw(link) as *mut L) },
            geometry,
            tx_buf,
            rx_buf,
        }
    }
}

impl<L: Link> Drop for Controller<L> {
    fn drop(&mut self) {
        if !self.link.is_open() {
            return;
        }
        let _ = self.close_impl(SenderOption::<SpinSleeper>::default());
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Mutex;

    use crate::{
        core::{
            defined::mm,
            derive::*,
            gain::{Gain, GainCalculator, GainCalculatorGenerator},
            link::LinkError,
        },
        driver::{
            autd3_device::AUTD3,
            datagram::{GainSTM, ReadsFPGAState},
            defined::Hz,
        },
        gain::Uniform,
        link::{Audit, AuditOption},
        modulation::Sine,
    };

    use super::*;

    // GRCOV_EXCL_START
    pub fn create_controller(dev_num: usize) -> anyhow::Result<Controller<Audit>> {
        Ok(Controller::open(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::new(AuditOption::default()),
        )?)
    }
    // GRCOV_EXCL_STOP

    #[derive(Gain, Debug)]
    pub struct TestGain {
        pub test: Arc<Mutex<Vec<bool>>>,
    }

    impl Gain for TestGain {
        type G = Null;

        // GRCOV_EXCL_START
        fn init(self) -> Result<Self::G, GainError> {
            unimplemented!()
        }
        // GRCOV_EXCL_STOP

        fn init_full(
            self,
            geometry: &Geometry,
            _filter: Option<&HashMap<usize, BitVec>>,
            _: bool,
        ) -> Result<Self::G, GainError> {
            geometry.iter().for_each(|dev| {
                self.test.lock().unwrap()[dev.idx()] = dev.enable;
            });
            Ok(Null {})
        }
    }

    #[test]
    fn open_failed() {
        assert_eq!(
            Some(AUTDError::Driver(LinkError::new("broken").into())),
            Controller::open(
                [AUTD3::default()],
                Audit::new(AuditOption {
                    broken: true,
                    ..Default::default()
                })
            )
            .err()
        );
    }

    #[test]
    fn send() -> anyhow::Result<()> {
        let mut autd = create_controller(1)?;
        autd.send((
            Sine {
                freq: 150. * Hz,
                option: Default::default(),
            },
            GainSTM {
                gains: vec![
                    Uniform {
                        intensity: EmitIntensity(0x80),
                        phase: Phase::ZERO,
                    },
                    Uniform {
                        intensity: EmitIntensity(0x81),
                        phase: Phase::ZERO,
                    },
                ],
                config: 1. * Hz,
                option: Default::default(),
            },
        ))?;

        autd.iter().try_for_each(|dev| {
            assert_eq!(
                *Sine {
                    freq: 150. * Hz,
                    option: Default::default(),
                }
                .calc()?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform {
                intensity: EmitIntensity(0x80),
                phase: Phase::ZERO,
            }
            .init()?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: EmitIntensity(0x81),
                phase: Phase::ZERO,
            }
            .init()?
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
            vec![FirmwareVersion {
                idx: 0,
                cpu: CPUVersion {
                    major: FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
                    minor: FirmwareVersion::LATEST_VERSION_NUM_MINOR
                },
                fpga: FPGAVersion {
                    major: FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
                    minor: FirmwareVersion::LATEST_VERSION_NUM_MINOR,
                    function_bits: FPGAVersion::ENABLED_EMULATOR_BIT
                }
            }],
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
            autd.close_impl(SenderOption::<SpinSleeper>::default())?;
            autd.close()?;
        }

        {
            let mut autd = create_controller(1)?;
            autd.link_mut().break_down();
            assert_eq!(
                Err(AUTDDriverError::Link(LinkError::new("broken"))),
                autd.close()
            );
        }

        Ok(())
    }

    #[test]
    fn fpga_state() -> anyhow::Result<()> {
        let mut autd = Controller::open(
            [AUTD3::default(), AUTD3::default()],
            Audit::new(AuditOption::default()),
        )?;

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
    fn with_boxed_link() -> anyhow::Result<()> {
        let link: Box<dyn Link> = Box::new(Audit::new(AuditOption::default()));
        let mut autd = Controller::open([AUTD3::default()], link)?;

        autd.send(Sine {
            freq: 150. * Hz,
            option: Default::default(),
        })?;

        autd.close()?;

        Ok(())
    }

    #[test]
    fn into_boxed_link_unsafe() -> anyhow::Result<()> {
        let option = SenderOption {
            sleeper: StdSleeper {
                timer_resolution: None,
            },
            ..Default::default()
        };
        let autd = Controller::open_with_option(
            [AUTD3::default()],
            Audit::new(AuditOption::default()),
            option,
        )?;

        let mut autd = autd.into_boxed_link();

        autd.sender(option).send((
            Sine {
                freq: 150. * Hz,
                option: Default::default(),
            },
            GainSTM {
                gains: vec![
                    Uniform {
                        intensity: EmitIntensity(0x80),
                        phase: Phase::ZERO,
                    },
                    Uniform {
                        intensity: EmitIntensity(0x81),
                        phase: Phase::ZERO,
                    },
                ],
                config: 1. * Hz,
                option: Default::default(),
            },
        ))?;

        let mut autd = unsafe { Controller::<Audit>::from_boxed_link(autd) };

        autd.iter().try_for_each(|dev| {
            assert_eq!(
                *Sine {
                    freq: 150. * Hz,
                    option: Default::default(),
                }
                .calc()?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform {
                intensity: EmitIntensity(0x80),
                phase: Phase::ZERO,
            }
            .init()?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: EmitIntensity(0x81),
                phase: Phase::ZERO,
            }
            .init()?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 1)
            );
            anyhow::Ok(())
        })?;

        autd.close_impl(option)?;

        Ok(())
    }

    #[test]
    fn into_boxed_link_close() -> anyhow::Result<()> {
        let autd = create_controller(1)?;
        let autd = autd.into_boxed_link();

        autd.close()?;

        Ok(())
    }
}
