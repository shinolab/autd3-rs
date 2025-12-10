use autd3_core::{
    datagram::{Datagram, DeviceMask},
    datagram::{Inspectable, InspectionResult},
    environment::Environment,
    link::{Ack, Link, MsgId, RxMessage},
    sleep::{Sleeper, StdSleeper},
};

use autd3_driver::{
    error::AUTDDriverError,
    firmware::{fpga::FPGAState, transmission::SenderOption, version::FirmwareVersion},
    geometry::{Device, Geometry},
};

/// A controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
pub struct Controller<L: Link> {
    link: L,
    geometry: Geometry,
    /// THe environment where the devices are placed.
    pub environment: Environment,
    msg_id: MsgId,
    sent_flags: Vec<bool>,
    rx_buf: Vec<RxMessage>,
    /// The default sender option used for [`send`](Controller::send).
    pub default_sender_option: SenderOption,
}

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: Link, S: Sleeper> {
    inner: autd3_driver::firmware::transmission::Sender<'a, L, S>,
}

impl<'a, L: Link, S: Sleeper> Sender<'a, L, S> {
    /// Send the [`Datagram`] to the devices.
    pub fn send<D: Datagram<'a>>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::operation::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::operation::Operation<'a>>::Error>,
    {
        self.inner.send(s)
    }
}

impl<L: Link> std::ops::Deref for Controller<L> {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl<L: Link> std::ops::DerefMut for Controller<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geometry
    }
}

impl<L: Link> Controller<L> {
    /// Equivalent to [`Self::open_with`] with default [`SenderOption`] and [`StdSleeper`].
    pub fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDDriverError> {
        Self::open_with(devices, link, Default::default(), StdSleeper)
    }

    /// Opens a controller with a [`SenderOption`].
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub fn open_with<D: Into<Device>, F: IntoIterator<Item = D>, S: Sleeper>(
        devices: F,
        mut link: L,
        option: SenderOption,
        sleeper: S,
    ) -> Result<Self, AUTDDriverError> {
        let geometry = Geometry::new(devices.into_iter().map(|d| d.into()).collect());
        let environment = Environment::default();

        link.open(&geometry)?;

        let mut cnt = Controller {
            link,
            msg_id: MsgId::new(0),
            sent_flags: vec![false; geometry.len()],
            rx_buf: vec![RxMessage::new(0, Ack::new(0x00, 0x00)); geometry.len()],
            geometry,
            environment,
            default_sender_option: option,
        };

        cnt.raw_sender(option, sleeper).initialize_devices()?;

        Ok(cnt)
    }

    #[doc(hidden)]
    pub const fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    #[doc(hidden)]
    pub fn geometry_mut(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    #[doc(hidden)]
    pub const fn link(&self) -> &L {
        &self.link
    }

    #[doc(hidden)]
    pub const fn link_mut(&mut self) -> &mut L {
        &mut self.link
    }

    /// Returns the [`Sender`] to send data to the devices.
    pub fn sender(&mut self, option: SenderOption) -> Sender<'_, L, StdSleeper> {
        self.sender_with_sleeper(option, StdSleeper)
    }

    /// Returns the [`Sender`] to send data to the devices with the given [`Sleeper`].
    pub fn sender_with_sleeper<S: Sleeper>(
        &mut self,
        option: SenderOption,
        sleeper: S,
    ) -> Sender<'_, L, S> {
        Sender {
            inner: self.raw_sender(option, sleeper),
        }
    }

    /// Sends a data to the devices. This is a shortcut for [`Sender::send`].
    ///
    /// [`Sender::send`]: autd3_driver::firmware::transmission::Sender::send
    pub fn send<'a, D: Datagram<'a>>(&'a mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::operation::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::operation::Operation<'a>>::Error>,
    {
        self.sender(self.default_sender_option).send(s)
    }

    /// Returns the inspection result.
    pub fn inspect<'a, I: Inspectable<'a>>(
        &'a self,
        s: I,
    ) -> Result<InspectionResult<I::Result>, I::Error> {
        s.inspect(&self.geometry, &self.environment, &DeviceMask::AllEnabled)
    }

    /// Closes the controller.
    pub fn close(mut self) -> Result<(), AUTDDriverError> {
        self.close_impl(self.default_sender_option)
    }

    /// Returns the firmware version of the devices.
    pub fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDDriverError> {
        self.raw_sender(self.default_sender_option, StdSleeper)
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
    pub fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.receive(&mut self.rx_buf)?;
        Ok(self.rx_buf.iter().map(FPGAState::from_rx).collect())
    }
}

impl<L: Link> Controller<L> {
    fn raw_sender<S: Sleeper>(
        &mut self,
        option: SenderOption,
        sleeper: S,
    ) -> autd3_driver::firmware::transmission::Sender<'_, L, S> {
        autd3_driver::firmware::transmission::Sender::new(
            &mut self.msg_id,
            &mut self.link,
            &self.geometry,
            &mut self.sent_flags,
            &mut self.rx_buf,
            &self.environment,
            option,
            sleeper,
        )
    }

    fn close_impl(&mut self, option: SenderOption) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Ok(());
        }
        self.raw_sender(option, StdSleeper).close()
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
        let environment = unsafe { std::ptr::read(&cnt.environment) };
        let sent_flags = unsafe { std::ptr::read(&cnt.sent_flags) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let default_sender_option = unsafe { std::ptr::read(&cnt.default_sender_option) };
        Controller {
            msg_id,
            link: Box::new(link) as _,
            geometry,
            environment,
            sent_flags,
            rx_buf,
            default_sender_option,
        }
    }

    /// Converts `Controller<Box<dyn Link>>` into a `Controller<L>`.
    ///
    /// # Safety
    ///
    /// This function must be used only when converting an instance created by [`Controller::into_boxed_link`] back to the original [`Controller`].
    pub unsafe fn from_boxed_link(cnt: Controller<Box<dyn Link>>) -> Controller<L> {
        let cnt = std::mem::ManuallyDrop::new(cnt);
        let msg_id = unsafe { std::ptr::read(&cnt.msg_id) };
        let link = unsafe { std::ptr::read(&cnt.link) };
        let geometry = unsafe { std::ptr::read(&cnt.geometry) };
        let environment = unsafe { std::ptr::read(&cnt.environment) };
        let sent_flags = unsafe { std::ptr::read(&cnt.sent_flags) };
        let rx_buf = unsafe { std::ptr::read(&cnt.rx_buf) };
        let default_sender_option = unsafe { std::ptr::read(&cnt.default_sender_option) };
        Controller {
            msg_id,
            link: unsafe { *Box::from_raw(Box::into_raw(link) as *mut L) },
            geometry,
            environment,
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
        let _ = self.close_impl(self.default_sender_option);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;

    use crate::{
        core::{
            devices::AUTD3,
            firmware::{Intensity, Phase, Segment},
            gain::{Gain, GainCalculator, GainCalculatorGenerator, TransducerMask},
            link::LinkError,
            modulation::{Modulation, ModulationInspectionResult},
        },
        driver::{
            common::Hz,
            datagram::{GainSTM, ReadsFPGAState},
        },
        gain::Uniform,
        link::{Audit, AuditOption},
        modulation::{Sine, Static},
    };

    use super::*;

    pub fn create_controller(dev_num: usize) -> Result<Controller<Audit>, AUTDDriverError> {
        Controller::open(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::new(AuditOption::default()),
        )
    }

    #[test]
    fn deref_mut() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = create_controller(1)?;
        assert_eq!(1, autd.len());
        autd.reconfigure(|dev| dev);
        Ok(())
    }

    #[test]
    fn geometry() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = create_controller(1)?;
        assert_eq!(1, autd.geometry().len());
        autd.geometry_mut().reconfigure(|dev| dev);
        Ok(())
    }

    #[test]
    fn open_failed() {
        assert_eq!(
            Some(AUTDDriverError::Link(LinkError::new("broken"))),
            Controller::open(
                [AUTD3::default()],
                Audit::new(AuditOption {
                    broken: true,
                    ..Default::default()
                }),
            )
            .err()
        );
    }

    #[test]
    fn send() -> Result<(), Box<dyn std::error::Error>> {
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
                .calc()?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform {
                intensity: Intensity(0x80),
                phase: Phase::ZERO,
            }
            .init(
                &autd.geometry,
                &autd.environment,
                &TransducerMask::AllEnabled,
            )?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: Intensity(0x81),
                phase: Phase::ZERO,
            }
            .init(
                &autd.geometry,
                &autd.environment,
                &TransducerMask::AllEnabled,
            )?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 1)
            );
            Result::<(), Box<dyn std::error::Error>>::Ok(())
        })?;

        autd.close()?;

        Ok(())
    }

    #[test]
    fn inspect() -> Result<(), Box<dyn std::error::Error>> {
        let autd = create_controller(2)?;

        let r = autd.inspect(autd3_driver::datagram::Group::new(
            |dev| (dev.idx() == 0).then_some(()),
            HashMap::from([((), Static::default())]),
        ))?;
        assert_eq!(autd.geometry.len(), r.len());
        assert_eq!(
            Some(ModulationInspectionResult {
                data: vec![0xFF, 0xFF],
                config: Static::default().sampling_config(),
            }),
            r[0]
        );
        assert_eq!(None, r[1]);

        autd.close()?;

        Ok(())
    }

    #[test]
    fn firmware_version() -> Result<(), Box<dyn std::error::Error>> {
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
    fn firmware_version_err() -> Result<(), Box<dyn std::error::Error>> {
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
    fn close() -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut autd = create_controller(1)?;
            autd.close_impl(SenderOption::default())?;
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
    fn fpga_state() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = Controller::open(
            [AUTD3::default(), AUTD3::default()],
            Audit::new(AuditOption::default()),
        )?;

        autd.send(ReadsFPGAState::new(|_| true))?;
        {
            autd.link_mut()[0].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state()?;
            assert_eq!(2, states.len());
            assert!(states[0].is_some_and(|s| s.is_thermal_assert()));
            assert!(states[1].is_some_and(|s| !s.is_thermal_assert()));
        }

        {
            autd.link_mut()[0].fpga_mut().deassert_thermal_sensor();
            autd.link_mut()[1].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state()?;
            assert_eq!(2, states.len());
            assert!(states[0].is_some_and(|s| !s.is_thermal_assert()));
            assert!(states[1].is_some_and(|s| s.is_thermal_assert()));
        }

        autd.send(ReadsFPGAState::new(|dev| dev.idx() == 1))?;
        {
            let states = autd.fpga_state()?;
            assert_eq!(2, states.len());
            assert!(states[0].is_none());
            assert!(states[1].is_some_and(|s| s.is_thermal_assert()));
        }

        Ok(())
    }

    #[test]
    fn into_iter() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = create_controller(1)?;
        (&mut autd).into_iter().for_each(|dev| {
            _ = dev;
        });
        (&autd).into_iter().for_each(|dev| {
            _ = dev;
        });
        Ok(())
    }

    #[test]
    fn with_boxed_link() -> Result<(), Box<dyn std::error::Error>> {
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
    fn into_boxed_link_unsafe() -> Result<(), Box<dyn std::error::Error>> {
        let autd = Controller::open_with(
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
                intensity: Intensity(0x80),
                phase: Phase::ZERO,
            }
            .init(
                &autd.geometry,
                &autd.environment,
                &TransducerMask::AllEnabled,
            )?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                intensity: Intensity(0x81),
                phase: Phase::ZERO,
            }
            .init(
                &autd.geometry,
                &autd.environment,
                &TransducerMask::AllEnabled,
            )?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 1)
            );
            Result::<(), Box<dyn std::error::Error>>::Ok(())
        })?;

        autd.close()?;

        Ok(())
    }

    #[test]
    fn into_boxed_link_close() -> Result<(), Box<dyn std::error::Error>> {
        let autd = create_controller(1)?;
        let autd = autd.into_boxed_link();

        autd.close()?;

        Ok(())
    }

    #[test]
    fn send_boxed() -> Result<(), Box<dyn std::error::Error>> {
        use crate::gain::Null;
        use autd3_driver::firmware::operation::BoxedDatagram;

        {
            let mut autd =
                Controller::open([AUTD3::default()], Audit::new(AuditOption::default()))?;

            autd.send(BoxedDatagram::new(Null))?;

            autd.close()?;
        }

        Ok(())
    }
}
