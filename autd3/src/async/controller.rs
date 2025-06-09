use crate::{
    controller::{FixedSchedule, SenderOption},
    gain::Null,
    modulation::Static,
};

use autd3_core::{
    datagram::{DeviceFilter, Inspectable, InspectionResult},
    link::{AsyncLink, MsgId},
};
use autd3_driver::{
    datagram::{Clear, Datagram, FixedCompletionSteps, Silencer},
    error::AUTDDriverError,
    firmware::{
        cpu::RxMessage,
        fpga::FPGAState,
        operation::{Operation, OperationGenerator},
        version::FirmwareVersion,
    },
    geometry::{Device, Geometry},
};

pub use autd3_core::sleep::r#async::{AsyncSleep, AsyncSleeper};
pub use autd3_driver::transmission::r#async::{AsyncTimerStrategy, Sender};

use derive_more::{Deref, DerefMut};
use getset::{Getters, MutGetters};

/// An asynchronous controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
#[derive(Deref, DerefMut, Getters, MutGetters)]
pub struct Controller<L: AsyncLink> {
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

impl<L: AsyncLink> Controller<L> {
    /// Equivalent to [`Self::open_with_option`] with default [`SenderOption`] and [`AsyncSleeper`].
    pub async fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Controller<L>, AUTDDriverError> {
        Self::open_with_option(
            devices,
            link,
            SenderOption::default(),
            FixedSchedule(AsyncSleeper),
        )
        .await
    }

    /// Opens a controller with a timeout.
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub async fn open_with_option<
        D: Into<Device>,
        F: IntoIterator<Item = D>,
        S: AsyncSleep,
        T: AsyncTimerStrategy<S>,
    >(
        devices: F,
        mut link: L,
        option: SenderOption,
        timer_strategy: T,
    ) -> Result<Self, AUTDDriverError> {
        let geometry = Geometry::new(devices.into_iter().map(|d| d.into()).collect());

        link.open(&geometry).await?;

        let mut cnt = Controller {
            link,
            sent_flags: smallvec::smallvec![false; geometry.len()],
            rx_buf: vec![RxMessage::new(0, 0); geometry.len()],
            msg_id: MsgId::new(0),
            geometry,
            default_sender_option: option,
        };

        cnt.sender(option, timer_strategy)
            .initialize_devices()
            .await?;

        Ok(cnt)
    }

    /// Returns the [`Sender`] to send data to the devices.
    pub fn sender<S: AsyncSleep, T: AsyncTimerStrategy<S>>(
        &mut self,
        option: SenderOption,
        timer_strategy: T,
    ) -> Sender<'_, L, S, T> {
        Sender::new(
            &mut self.msg_id,
            &mut self.link,
            &mut self.geometry,
            &mut self.sent_flags,
            &mut self.rx_buf,
            option,
            timer_strategy,
        )
    }

    /// Sends a data to the devices. This is a shortcut for [`Sender::send`].
    pub async fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .send(s)
            .await
    }

    /// Returns the inspection result.
    pub fn inspect<I: Inspectable>(
        &mut self,
        s: I,
    ) -> Result<InspectionResult<I::Result>, I::Error> {
        s.inspect(&self.geometry, &DeviceFilter::all_enabled())
    }

    /// Closes the controller.
    pub async fn close(mut self) -> Result<(), AUTDDriverError> {
        self.close_impl(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .await
    }

    /// Returns the firmware version of the devices.
    pub async fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDDriverError> {
        self.sender(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .firmware_version()
            .await
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
    pub async fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.receive(&mut self.rx_buf).await?;
        Ok(self.rx_buf.iter().map(FPGAState::from_rx).collect())
    }
}

impl<L: AsyncLink> Controller<L> {
    async fn close_impl<S: AsyncSleep, T: AsyncTimerStrategy<S>>(
        &mut self,
        option: SenderOption,
        timer_strategy: T,
    ) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Ok(());
        }

        let mut sender = self.sender(option, timer_strategy);

        [
            sender
                .send(Silencer {
                    config: FixedCompletionSteps {
                        strict_mode: false,
                        ..Default::default()
                    },
                })
                .await,
            sender.send((Static::default(), Null)).await,
            sender.send(Clear {}).await,
            Ok(self.link.close().await?),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }
}

impl<'a, L: AsyncLink> IntoIterator for &'a Controller<L> {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter()
    }
}

impl<'a, L: AsyncLink> IntoIterator for &'a mut Controller<L> {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter_mut()
    }
}

#[cfg(feature = "async-trait")]
impl<L: AsyncLink + 'static> Controller<L> {
    /// Converts `Controller<L>` into a `Controller<Box<dyn Link>>`.
    pub fn into_boxed_link(self) -> Controller<Box<dyn AsyncLink>> {
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
    pub unsafe fn from_boxed_link(cnt: Controller<Box<dyn AsyncLink>>) -> Controller<L> {
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

impl<L: AsyncLink> Drop for Controller<L> {
    fn drop(&mut self) {
        if !self.link.is_open() {
            return;
        }
        match tokio::runtime::Handle::current().runtime_flavor() {
            tokio::runtime::RuntimeFlavor::CurrentThread => {}
            tokio::runtime::RuntimeFlavor::MultiThread => tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = self
                        .close_impl(self.default_sender_option, FixedSchedule(AsyncSleeper))
                        .await;
                });
            }),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use autd3_core::{
        common::mm,
        derive::{Modulation, Segment},
        gain::{
            EmitIntensity, Gain, GainCalculator, GainCalculatorGenerator, Phase, TransducerFilter,
        },
        link::LinkError,
    };
    use autd3_driver::{
        autd3_device::AUTD3,
        common::Hz,
        datagram::{GainSTM, ReadsFPGAState},
    };

    use crate::{
        gain::Uniform,
        link::{Audit, AuditOption},
        modulation::Sine,
    };

    use super::*;

    // GRCOV_EXCL_START
    pub async fn create_controller(dev_num: usize) -> anyhow::Result<Controller<Audit>> {
        Ok(Controller::open(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::new(AuditOption::default()),
        )
        .await?)
    }
    // GRCOV_EXCL_STOP

    #[tokio::test]
    async fn open_failed() {
        assert_eq!(
            Some(AUTDDriverError::Link(LinkError::new("broken"))),
            Controller::open(
                [AUTD3::default()],
                Audit::new(AuditOption {
                    broken: true,
                    ..Default::default()
                })
            )
            .await
            .err()
        );
    }

    #[tokio::test]
    async fn send() -> anyhow::Result<()> {
        let mut autd = create_controller(1).await?;
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
        ))
        .await?;

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

        autd.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn inspect() -> anyhow::Result<()> {
        use crate::core::derive::ModulationInspectionResult;
        use crate::prelude::LoopBehavior;

        let mut autd = create_controller(2).await?;

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

        autd.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn firmware_version() -> anyhow::Result<()> {
        use autd3_driver::firmware::version::{CPUVersion, FPGAVersion};

        let mut autd = create_controller(1).await?;
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
            autd.firmware_version().await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn firmware_version_err() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;
        autd.link_mut().break_down();
        assert_eq!(
            Err(AUTDDriverError::ReadFirmwareVersionFailed(vec![
                false, false
            ])),
            autd.firmware_version().await
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn close() -> anyhow::Result<()> {
        {
            let mut autd = create_controller(1).await?;
            autd.close_impl(SenderOption::default(), FixedSchedule(AsyncSleeper))
                .await?;
            autd.close().await?;
        }

        {
            let mut autd = create_controller(1).await?;
            autd.link_mut().break_down();
            assert_eq!(
                Err(AUTDDriverError::Link(LinkError::new("broken"))),
                autd.close().await
            );
        }

        {
            _ = create_controller(1).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn fpga_state() -> anyhow::Result<()> {
        let mut autd = Controller::open(
            [AUTD3::default(), AUTD3::default()],
            Audit::new(AuditOption::default()),
        )
        .await?;

        autd.send(ReadsFPGAState::new(|_| true)).await?;
        {
            autd.link_mut()[0].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state().await?;
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

            let states = autd.fpga_state().await?;
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

        autd.send(ReadsFPGAState::new(|dev| dev.idx() == 1)).await?;
        {
            let states = autd.fpga_state().await?;
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

    #[tokio::test]
    async fn into_iter() -> anyhow::Result<()> {
        let mut autd = create_controller(1).await?;

        for dev in &mut autd {
            dev.sound_speed = 300e3 * mm;
        }

        for dev in &autd {
            assert_eq!(300e3 * mm, dev.sound_speed);
        }

        Ok(())
    }

    #[cfg(feature = "async-trait")]
    #[tokio::test]
    async fn into_boxed_link() -> anyhow::Result<()> {
        let autd = create_controller(1).await?;

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
        ))
        .await?;

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

        autd.close().await?;

        Ok(())
    }

    #[cfg(feature = "async-trait")]
    #[tokio::test]
    async fn into_boxed_link_close() -> anyhow::Result<()> {
        let autd = create_controller(1).await?;
        let autd = autd.into_boxed_link();

        autd.close().await?;

        Ok(())
    }
}
