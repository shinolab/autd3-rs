use autd3_core::{
    datagram::{Inspectable, InspectionResult},
    derive::{Datagram, DeviceFilter},
    link::{Ack, Link, MsgId, RxMessage},
    sleep::Sleep,
};
pub use autd3_driver::firmware::driver::{
    Driver, FPGAState, FixedDelay, FixedSchedule, ParallelMode, SenderOption, TimerStrategy,
};
use autd3_driver::{
    error::AUTDDriverError,
    firmware::{self, auto::Auto, driver::Sender, version::FirmwareVersion},
    geometry::{Device, Geometry},
};

use derive_more::{Deref, DerefMut};
use getset::{Getters, MutGetters};

/// A controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
#[derive(Deref, DerefMut, Getters, MutGetters)]
pub struct Controller<L: Link, V: Driver> {
    /// The link to the devices.
    #[getset(get = "pub", get_mut = "pub")]
    link: L,
    #[doc(hidden)]
    #[getset(get = "pub")]
    driver: V,
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

impl<L: Link> Controller<L, Auto> {
    /// Equivalent to [`Self::open_with_option`] with default [`SenderOption`], [`FixedSchedule`] and [`Auto`] diver.
    pub fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDDriverError> {
        Self::open_with_option(devices, link, Default::default(), FixedSchedule::default())
    }
}

impl<L: Link, V: Driver> Controller<L, V> {
    /// Equivalent to [`Self::open_with_option`] with default [`SenderOption`] and [`FixedSchedule`].
    pub fn open_with<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDDriverError> {
        Self::open_with_option(devices, link, Default::default(), FixedSchedule::default())
    }

    /// Opens a controller with a [`SenderOption`].
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub fn open_with_option<
        D: Into<Device>,
        F: IntoIterator<Item = D>,
        S: Sleep,
        T: TimerStrategy<S>,
    >(
        devices: F,
        mut link: L,
        option: SenderOption,
        timer_strategy: T,
    ) -> Result<Self, AUTDDriverError> {
        let geometry = Geometry::new(devices.into_iter().map(|d| d.into()).collect());

        link.open(&geometry)?;

        let mut msg_id = MsgId::new(0);
        let mut sent_flags = smallvec::smallvec![false; geometry.len()];
        let mut rx_buf = vec![RxMessage::new(0, Ack::new()); geometry.len()];

        let mut driver = V::new();
        driver.detect_version(
            &mut msg_id,
            &mut link,
            &geometry,
            &mut sent_flags,
            &mut rx_buf,
        )?;

        let mut cnt = Controller {
            link,
            driver,
            msg_id,
            sent_flags,
            rx_buf,
            geometry,
            default_sender_option: option,
        };

        cnt.sender(option, timer_strategy).initialize_devices()?;

        Ok(cnt)
    }

    /// Returns the [`Sender`] to send data to the devices.
    pub fn sender<S: Sleep, T: TimerStrategy<S>>(
        &mut self,
        option: SenderOption,
        timer_strategy: T,
    ) -> V::Sender<'_, L, S, T> {
        self.driver.sender(
            &mut self.msg_id,
            &mut self.link,
            &self.geometry,
            &mut self.sent_flags,
            &mut self.rx_buf,
            option,
            timer_strategy,
        )
    }

    /// Returns the inspection result.
    pub fn inspect<I: Inspectable>(&self, s: I) -> Result<InspectionResult<I::Result>, I::Error> {
        s.inspect(
            &self.geometry,
            &DeviceFilter::all_enabled(),
            &self.driver.firmware_limits(),
        )
    }

    /// Closes the controller.
    pub fn close(mut self) -> Result<(), AUTDDriverError> {
        self.close_impl(self.default_sender_option, FixedSchedule::default())
    }

    /// Returns the firmware version of the devices.
    pub fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDDriverError> {
        self.sender(self.default_sender_option, FixedSchedule::default())
            .firmware_version()
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
    /// # fn main() -> Result<(), AUTDDriverError> {
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
    pub fn fpga_state(&mut self) -> Result<Vec<Option<V::FPGAState>>, AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.receive(&mut self.rx_buf)?;
        Ok(self.rx_buf.iter().map(V::FPGAState::from_rx).collect())
    }
}

impl<L: Link, V: Driver> Controller<L, V> {
    fn close_impl<S: Sleep, T: TimerStrategy<S>>(
        &mut self,
        option: SenderOption,
        timer_strategy: T,
    ) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Ok(());
        }

        self.sender(option, timer_strategy).close()
    }
}

impl<'a, L: Link, V: Driver> IntoIterator for &'a Controller<L, V> {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter()
    }
}

impl<'a, L: Link, V: Driver> IntoIterator for &'a mut Controller<L, V> {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter_mut()
    }
}

impl<L: Link + 'static, V: Driver> Controller<L, V> {
    /// Converts `Controller<L>` into a `Controller<Box<dyn Link>>`.
    pub fn into_boxed_link(self) -> Controller<Box<dyn Link>, V> {
        let cnt = std::mem::ManuallyDrop::new(self);
        let msg_id = unsafe { std::ptr::read(&cnt.msg_id) };
        let driver = unsafe { std::ptr::read(&cnt.driver) };
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let sent_flags = unsafe { std::ptr::read(&cnt.sent_flags) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let default_sender_option = unsafe { std::ptr::read(&cnt.default_sender_option) };
        Controller {
            msg_id,
            driver,
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
    pub unsafe fn from_boxed_link(cnt: Controller<Box<dyn Link>, V>) -> Controller<L, V> {
        let cnt = std::mem::ManuallyDrop::new(cnt);
        let msg_id = unsafe { std::ptr::read(&cnt.msg_id) };
        let driver = unsafe { std::ptr::read(&cnt.driver) };
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let sent_flags = unsafe { std::ptr::read(&cnt.sent_flags) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let default_sender_option = unsafe { std::ptr::read(&cnt.default_sender_option) };
        Controller {
            msg_id,
            driver,
            link: unsafe { *Box::from_raw(Box::into_raw(link) as *mut L) },
            geometry,
            sent_flags,
            rx_buf,
            default_sender_option,
        }
    }
}

impl<L: Link, V: Driver> Drop for Controller<L, V> {
    fn drop(&mut self) {
        if !self.link.is_open() {
            return;
        }
        let _ = self.close_impl(self.default_sender_option, FixedSchedule::default());
    }
}

// The following implementations are necessary because Rust does not have associated traits.
// https://github.com/rust-lang/rfcs/issues/2190

impl<L: Link> Controller<L, firmware::v12::V12> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::v12::transmission::Sender`].
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::v12::operation::OperationGenerator,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::v12::operation::OperationGenerator>::O1 as autd3_driver::firmware::v12::operation::Operation>::Error>
            + From<<<D::G as autd3_driver::firmware::v12::operation::OperationGenerator>::O2 as autd3_driver::firmware::v12::operation::Operation>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule::default())
            .send(s)
    }
}

impl<L: Link> Controller<L, firmware::v11::V11> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::v11::transmission::Sender`].
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::v11::operation::OperationGenerator,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::v11::operation::OperationGenerator>::O1 as autd3_driver::firmware::v11::operation::Operation>::Error>
            + From<<<D::G as autd3_driver::firmware::v11::operation::OperationGenerator>::O2 as autd3_driver::firmware::v11::operation::Operation>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule::default())
            .send(s)
    }
}

impl<L: Link> Controller<L, firmware::v10::V10> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::v10::transmission::Sender`].
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::v10::operation::OperationGenerator,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::v10::operation::OperationGenerator>::O1 as autd3_driver::firmware::v10::operation::Operation>::Error>
            + From<<<D::G as autd3_driver::firmware::v10::operation::OperationGenerator>::O2 as autd3_driver::firmware::v10::operation::Operation>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule::default())
            .send(s)
    }
}

impl<L: Link> Controller<L, firmware::auto::Auto> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::auto::transmission::Sender`].
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::auto::operation::OperationGenerator,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::auto::operation::OperationGenerator>::O1 as autd3_driver::firmware::auto::operation::Operation>::Error>
            + From<<<D::G as autd3_driver::firmware::auto::operation::OperationGenerator>::O2 as autd3_driver::firmware::auto::operation::Operation>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule::default())
            .send(s)
    }
}

impl<L: Link> Controller<L, firmware::v12::V12> {
    /// Make a [`firmware::v12::operation::BoxedDatagram`].
    pub fn make_boxed<
        E,
        G: firmware::v12::operation::DOperationGenerator + 'static,
        D: Datagram<G = G, Error = E> + 'static,
    >(
        &self,
        d: D,
    ) -> firmware::v12::operation::BoxedDatagram
    where
        AUTDDriverError: From<E>,
    {
        firmware::v12::operation::BoxedDatagram::new(d)
    }
}

impl<L: Link> Controller<L, firmware::v11::V11> {
    /// Make a [`firmware::v11::operation::BoxedDatagram`].
    pub fn make_boxed<
        E,
        G: firmware::v11::operation::DOperationGenerator + 'static,
        D: Datagram<G = G, Error = E> + 'static,
    >(
        &self,
        d: D,
    ) -> firmware::v11::operation::BoxedDatagram
    where
        AUTDDriverError: From<E>,
    {
        firmware::v11::operation::BoxedDatagram::new(d)
    }
}

impl<L: Link> Controller<L, firmware::v10::V10> {
    /// Make a [`firmware::v10::operation::BoxedDatagram`].
    pub fn make_boxed<
        E,
        G: firmware::v10::operation::DOperationGenerator + 'static,
        D: Datagram<G = G, Error = E> + 'static,
    >(
        &self,
        d: D,
    ) -> firmware::v10::operation::BoxedDatagram
    where
        AUTDDriverError: From<E>,
    {
        firmware::v10::operation::BoxedDatagram::new(d)
    }
}

impl<L: Link> Controller<L, firmware::auto::Auto> {
    /// Make a [`firmware::auto::operation::BoxedDatagram`].
    pub fn make_boxed<
        E,
        G: firmware::auto::operation::DOperationGenerator + 'static,
        D: Datagram<G = G, Error = E> + 'static,
    >(
        &self,
        d: D,
    ) -> firmware::auto::operation::BoxedDatagram
    where
        AUTDDriverError: From<E>,
    {
        firmware::auto::operation::BoxedDatagram::new(d)
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
            firmware,
            firmware::latest::Latest,
        },
        gain::Uniform,
        link::{Audit, AuditOption, audit::version},
        modulation::{Sine, Static},
    };

    use super::*;

    pub fn create_controller(
        dev_num: usize,
    ) -> anyhow::Result<Controller<Audit<version::Latest>, Latest>> {
        Ok(Controller::open_with(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::latest(AuditOption::default()),
        )?)
    }

    #[test]
    fn open_failed() {
        assert_eq!(
            Some(AUTDDriverError::Link(LinkError::new("broken"))),
            Controller::<_, Latest>::open_with(
                [AUTD3::default()],
                Audit::latest(AuditOption {
                    broken: true,
                    ..Default::default()
                }),
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
                        intensity: Intensity(0x80),
                        phase: Phase::ZERO,
                    },
                    Uniform {
                        intensity: Intensity(0x81),
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
                .calc(&Latest.firmware_limits())?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform {
                intensity: Intensity(0x80),
                phase: Phase::ZERO,
            }
            .init(&autd.geometry, &TransducerFilter::all_enabled())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: Intensity(0x81),
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
            Err(AUTDDriverError::ReadFirmwareVersionFailed(vec![
                false, false
            ])),
            autd.firmware_version()
        );
        Ok(())
    }

    #[test]
    fn close() -> anyhow::Result<()> {
        {
            let mut autd = create_controller(1)?;
            autd.close_impl(SenderOption::default(), FixedSchedule::default())?;
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
        let mut autd = Controller::<_, Latest>::open_with(
            [AUTD3::default(), AUTD3::default()],
            Audit::latest(AuditOption::default()),
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
        let link: Box<dyn Link> = Box::new(Audit::latest(AuditOption::default()));
        let mut autd = Controller::<_, Latest>::open_with([AUTD3::default()], link)?;

        autd.send(Sine {
            freq: 150. * Hz,
            option: Default::default(),
        })?;

        autd.close()?;

        Ok(())
    }

    #[test]
    fn into_boxed_link_unsafe() -> anyhow::Result<()> {
        let autd = Controller::<_, Latest>::open_with_option(
            [AUTD3::default()],
            Audit::latest(AuditOption::default()),
            SenderOption::default(),
            FixedSchedule::default(),
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
                        intensity: Intensity(0x80),
                        phase: Phase::ZERO,
                    },
                    Uniform {
                        intensity: Intensity(0x81),
                        phase: Phase::ZERO,
                    },
                ],
                config: 1. * Hz,
                option: Default::default(),
            },
        ))?;

        let autd = unsafe { Controller::<Audit<version::Latest>, _>::from_boxed_link(autd) };

        autd.iter().try_for_each(|dev| {
            assert_eq!(
                *Sine {
                    freq: 150. * Hz,
                    option: Default::default(),
                }
                .calc(&Latest.firmware_limits())?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform {
                intensity: Intensity(0x80),
                phase: Phase::ZERO,
            }
            .init(&autd.geometry, &TransducerFilter::all_enabled())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: Intensity(0x81),
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

    #[test]
    fn make_boxed() -> anyhow::Result<()> {
        use crate::gain::Null;

        {
            let mut autd = Controller::<_, firmware::v12::V12>::open_with(
                [AUTD3::default()],
                Audit::<version::V12>::new(AuditOption::default()),
            )?;

            autd.send(autd.make_boxed(Null))?;

            autd.close()?;
        }

        {
            let mut autd = Controller::<_, firmware::v11::V11>::open_with(
                [AUTD3::default()],
                Audit::<version::V11>::new(AuditOption::default()),
            )?;

            autd.send(autd.make_boxed(Null))?;

            autd.close()?;
        }

        {
            let mut autd = Controller::<_, firmware::v10::V10>::open_with(
                [AUTD3::default()],
                Audit::<version::V10>::new(AuditOption::default()),
            )?;

            autd.send(autd.make_boxed(Null))?;

            autd.close()?;
        }

        {
            let mut autd = Controller::<_, firmware::auto::Auto>::open_with(
                [AUTD3::default()],
                Audit::<version::Latest>::new(AuditOption::default()),
            )?;

            autd.send(autd.make_boxed(Null))?;

            autd.close()?;
        }

        Ok(())
    }
}
