mod sender;

use crate::{error::AUTDError, gain::Null, modulation::Static};

use autd3_core::{
    datagram::{Inspectable, InspectionResult},
    derive::DeviceFilter,
    link::{Link, MsgId},
};
use autd3_driver::{
    datagram::{Clear, Datagram, FixedCompletionSteps, ReadsFPGAState, Silencer, Synchronize},
    error::AUTDDriverError,
    firmware::{
        cpu::{RxMessage, check_if_msg_is_processed},
        fpga::FPGAState,
        operation::{FirmwareVersionType, Operation, OperationGenerator},
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry},
};

pub use sender::{
    ParallelMode, Sender, SenderOption, SpinSleeper, SpinStrategy, StdSleeper, sleep::Sleep,
};

use derive_more::{Deref, DerefMut};
use getset::{Getters, MutGetters};
use tracing;

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
    msg_id: MsgId,
    sent_flags: smallvec::SmallVec<[bool; 32]>,
    rx_buf: Vec<RxMessage>,
    /// The default sender option used for [`send`](Controller::send).
    pub default_sender_option: SenderOption,
}

impl<L: Link> Controller<L> {
    /// Equivalent to [`Self::open_with_option`] with default [`SenderOption`] and [`SpinSleeper`].
    pub fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDError> {
        Self::open_with_option(
            devices,
            link,
            SenderOption::default(),
            SpinSleeper::default(),
        )
    }

    /// Opens a controller with a [`SenderOption`].
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub fn open_with_option<D: Into<Device>, F: IntoIterator<Item = D>, S: Sleep>(
        devices: F,
        mut link: L,
        option: SenderOption,
        sleeper: S,
    ) -> Result<Self, AUTDError> {
        tracing::debug!("Opening a controller with option {:?})", option);

        let geometry = Geometry::new(devices.into_iter().map(|d| d.into()).collect());

        link.open(&geometry)?;

        let mut cnt = Controller {
            link,
            msg_id: MsgId::new(0),
            sent_flags: smallvec::smallvec![false; geometry.len()],
            rx_buf: vec![RxMessage::new(0, 0); geometry.len()],
            geometry,
            default_sender_option: option,
        };

        let mut sender = cnt.sender(option, sleeper);

        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data (here, we use `ReadsFPGAState` because it is the lightest).
        let _ = sender.send(ReadsFPGAState::new(|_| false));

        sender.send((Clear::new(), Synchronize::new()))?;

        Ok(cnt)
    }

    /// Returns the [`Sender`] to send data to the devices.
    pub fn sender<S: Sleep>(&mut self, option: SenderOption, sleeper: S) -> Sender<'_, L, S> {
        Sender {
            msg_id: &mut self.msg_id,
            link: &mut self.link,
            geometry: &mut self.geometry,
            sent_flags: &mut self.sent_flags,
            rx: &mut self.rx_buf,
            option,
            sleeper,
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
        self.sender(self.default_sender_option, SpinSleeper::default())
            .send(s)
    }

    /// Returns the inspection result.
    pub fn inspect<I: Inspectable>(&self, s: I) -> Result<InspectionResult<I::Result>, I::Error> {
        s.inspect(&self.geometry, &DeviceFilter::all_enabled())
    }

    /// Closes the controller.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn close(mut self) -> Result<(), AUTDDriverError> {
        self.close_impl(self.default_sender_option, SpinSleeper::default())
    }

    /// Returns the firmware version of the devices.
    pub fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDError> {
        use FirmwareVersionType::*;
        use autd3_driver::firmware::version::{CPUVersion, FPGAVersion, Major, Minor};

        let cpu_major = self.fetch_firminfo(CPUMajor)?;
        let cpu_minor = self.fetch_firminfo(CPUMinor)?;
        let fpga_major = self.fetch_firminfo(FPGAMajor)?;
        let fpga_minor = self.fetch_firminfo(FPGAMinor)?;
        let fpga_functions = self.fetch_firminfo(FPGAFunctions)?;
        self.fetch_firminfo(Clear)?;

        Ok(self
            .geometry
            .iter()
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

impl<L: Link> Controller<L> {
    fn close_impl<S: Sleep>(
        &mut self,
        option: SenderOption,
        sleeper: S,
    ) -> Result<(), AUTDDriverError> {
        tracing::info!("Closing controller");

        if !self.link.is_open() {
            tracing::warn!("Link is already closed");
            return Ok(());
        }

        let mut sender = self.sender(option, sleeper);

        [
            sender.send(Silencer {
                config: FixedCompletionSteps {
                    strict_mode: false,
                    ..Default::default()
                },
            }),
            sender.send((Static::default(), Null)),
            sender.send(Clear {}),
            Ok(self.link.close()?),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }

    fn fetch_firminfo(&mut self, ty: FirmwareVersionType) -> Result<Vec<u8>, AUTDError> {
        self.send(ty).map_err(|e| {
            tracing::error!("Fetch firmware info failed: {:?}", e);
            AUTDError::ReadFirmwareVersionFailed(
                check_if_msg_is_processed(self.msg_id, &self.rx_buf).collect(),
            )
        })?;
        Ok(self.rx_buf.iter().map(|rx| rx.data()).collect())
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
        let msg_id = unsafe { std::ptr::read(&cnt.msg_id) };
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let sent_flags = unsafe { std::ptr::read(&cnt.sent_flags) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let default_sender_option = unsafe { std::ptr::read(&cnt.default_sender_option) };
        Controller {
            msg_id,
            link: Box::new(link) as _,
            geometry,
            sent_flags,
            rx_buf,
            default_sender_option,
        }
    }

    /// Converts `Controller<Box<dyn Link>>` into a `Controller<L>`.
    ///
    /// # Safety
    ///
    /// This function must be used only when converting an instance created by [`Controller::into_boxed_link`] back to the original [`Controller<L>`].
    pub unsafe fn from_boxed_link(cnt: Controller<Box<dyn Link>>) -> Controller<L> {
        let cnt = std::mem::ManuallyDrop::new(cnt);
        let msg_id = unsafe { std::ptr::read(&cnt.msg_id) };
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let sent_flags = unsafe { std::ptr::read(&cnt.sent_flags) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let default_sender_option = unsafe { std::ptr::read(&cnt.default_sender_option) };
        Controller {
            msg_id,
            link: unsafe { *Box::from_raw(Box::into_raw(link) as *mut L) },
            geometry,
            sent_flags,
            rx_buf,
            default_sender_option,
        }
    }
}

impl<L: Link> Drop for Controller<L> {
    fn drop(&mut self) {
        if !self.link.is_open() {
            return;
        }
        let _ = self.close_impl(self.default_sender_option, SpinSleeper::default());
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        core::{
            common::mm,
            derive::*,
            gain::{Gain, GainCalculator, GainCalculatorGenerator},
            link::LinkError,
        },
        driver::{
            autd3_device::AUTD3,
            common::Hz,
            datagram::{GainSTM, ReadsFPGAState},
        },
        gain::Uniform,
        link::{Audit, AuditOption},
        modulation::Sine,
    };

    use super::*;

    pub fn create_controller(dev_num: usize) -> anyhow::Result<Controller<Audit>> {
        Ok(Controller::open(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::new(AuditOption::default()),
        )?)
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
            .init(&autd.geometry, &TransducerFilter::all_enabled())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: EmitIntensity(0x81),
                phase: Phase::ZERO,
            }
            .init(&autd.geometry, &TransducerFilter::all_enabled())?
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
    fn inspect() -> anyhow::Result<()> {
        let autd = create_controller(2)?;

        let r = autd.inspect(autd3_driver::datagram::Group::new(
            |dev| (dev.idx() == 0).then_some(()),
            HashMap::from([((), Static::default())]),
        ))?;
        assert_eq!(autd.geometry.len(), r.len());
        assert_eq!(
            Some(ModulationInspectionResult {
                name: "Static".to_string(),
                data: vec![0xFF, 0xFF],
                config: Static::default().sampling_config(),
                loop_behavior: LoopBehavior::Infinite,
                segment: Segment::S0,
                transition_mode: None
            }),
            r[0]
        );
        assert_eq!(None, r[1]);

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
            autd.close_impl(SenderOption::default(), SpinSleeper::default())?;
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
            assert!(
                states[0]
                    .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                    .is_thermal_assert()
            );
            assert!(
                !states[1]
                    .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                    .is_thermal_assert()
            );
        }

        {
            autd.link_mut()[0].fpga_mut().deassert_thermal_sensor();
            autd.link_mut()[1].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state()?;
            assert_eq!(2, states.len());
            assert!(
                !states[0]
                    .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                    .is_thermal_assert()
            );
            assert!(
                states[1]
                    .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                    .is_thermal_assert()
            );
        }

        autd.send(ReadsFPGAState::new(|dev| dev.idx() == 1))?;
        {
            let states = autd.fpga_state()?;
            assert_eq!(2, states.len());
            assert!(states[0].is_none());
            assert!(
                states[1]
                    .ok_or(anyhow::anyhow!("state shouldn't be None here"))?
                    .is_thermal_assert()
            );
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
        let autd = Controller::open_with_option(
            [AUTD3::default()],
            Audit::new(AuditOption::default()),
            SenderOption::default(),
            StdSleeper,
        )?;

        let mut autd = autd.into_boxed_link();

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

        let autd = unsafe { Controller::<Audit>::from_boxed_link(autd) };

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
            .init(&autd.geometry, &TransducerFilter::all_enabled())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: EmitIntensity(0x81),
                phase: Phase::ZERO,
            }
            .init(&autd.geometry, &TransducerFilter::all_enabled())?
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
    fn into_boxed_link_close() -> anyhow::Result<()> {
        let autd = create_controller(1)?;
        let autd = autd.into_boxed_link();

        autd.close()?;

        Ok(())
    }
}
