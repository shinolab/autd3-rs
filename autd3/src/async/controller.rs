use autd3_core::{
    datagram::{Datagram, DeviceFilter, Inspectable, InspectionResult},
    environment::Environment,
    link::{Ack, AsyncLink, MsgId, RxMessage},
    sleep::r#async::Sleep,
};

pub use autd3_driver::firmware::driver::{
    FPGAState, FixedDelay, FixedSchedule, ParallelMode, SenderOption,
    r#async::{Driver, TimerStrategy},
};
use autd3_driver::{
    error::AUTDDriverError,
    firmware::{self, auto::Auto, driver::r#async::Sender, version::FirmwareVersion},
    geometry::{Device, Geometry},
};

use derive_more::{Deref, DerefMut};
use getset::{Getters, MutGetters};

pub use autd3_core::sleep::r#async::AsyncSleeper;

/// An asynchronous controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
#[derive(Deref, DerefMut, Getters, MutGetters)]
pub struct Controller<L: AsyncLink, V: Driver> {
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
    /// THe environment where the devices are placed.
    pub environment: Environment,
    msg_id: MsgId,
    sent_flags: smallvec::SmallVec<[bool; 32]>,
    rx_buf: Vec<RxMessage>,
    /// The default sender option used for [`send`](Controller::send).
    pub default_sender_option: SenderOption,
}

impl<L: AsyncLink> Controller<L, Auto> {
    /// Equivalent to [`Self::open_with_option`] with default [`SenderOption`], [`FixedSchedule`] and [`Auto`] diver.
    pub async fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDDriverError> {
        Self::open_with_option(
            devices,
            link,
            Default::default(),
            FixedSchedule(AsyncSleeper),
        )
        .await
    }
}

impl<L: AsyncLink, V: Driver> Controller<L, V> {
    /// Equivalent to [`Self::open_with_option`] with default [`SenderOption`] and [`FixedSchedule`].
    pub async fn open_with<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDDriverError> {
        Self::open_with_option(
            devices,
            link,
            Default::default(),
            FixedSchedule(AsyncSleeper),
        )
        .await
    }

    /// Opens a controller with a [`SenderOption`].
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub async fn open_with_option<
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
        let environment = Environment::default();

        link.open(&geometry).await?;

        let mut msg_id = MsgId::new(0);
        let mut sent_flags = smallvec::smallvec![false; geometry.len()];
        let mut rx_buf = vec![RxMessage::new(0, Ack::new()); geometry.len()];

        let mut driver = V::new();
        driver
            .detect_version(
                &mut msg_id,
                &mut link,
                &geometry,
                &mut sent_flags,
                &mut rx_buf,
                &environment,
            )
            .await?;

        let mut cnt = Controller {
            link,
            driver,
            msg_id,
            sent_flags,
            rx_buf,
            geometry,
            environment,
            default_sender_option: option,
        };

        cnt.sender(option, timer_strategy)
            .initialize_devices()
            .await?;

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
            &self.environment,
            option,
            timer_strategy,
        )
    }

    /// Returns the inspection result.
    pub fn inspect<'a, I: Inspectable<'a>>(
        &'a self,
        s: I,
    ) -> Result<InspectionResult<I::Result>, I::Error> {
        s.inspect(
            &self.geometry,
            &self.environment,
            &DeviceFilter::all_enabled(),
            &self.driver.firmware_limits(),
        )
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
    pub async fn fpga_state(&mut self) -> Result<Vec<Option<V::FPGAState>>, AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.receive(&mut self.rx_buf).await?;
        Ok(self.rx_buf.iter().map(V::FPGAState::from_rx).collect())
    }
}

impl<L: AsyncLink, V: Driver> Controller<L, V> {
    async fn close_impl<S: Sleep, T: TimerStrategy<S>>(
        &mut self,
        option: SenderOption,
        timer_strategy: T,
    ) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Ok(());
        }

        self.sender(option, timer_strategy).close().await
    }
}

impl<'a, L: AsyncLink, V: Driver> IntoIterator for &'a Controller<L, V> {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter()
    }
}

impl<'a, L: AsyncLink, V: Driver> IntoIterator for &'a mut Controller<L, V> {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter_mut()
    }
}

// The following implementations are necessary because Rust does not have associated traits.
// https://github.com/rust-lang/rfcs/issues/2190

impl<L: AsyncLink> Controller<L, firmware::v12_1::V12_1> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::v12_1::transmission::Sender`].
    pub async fn send<'a, D: Datagram<'a>>(&'a mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::v12_1::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::v12_1::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::driver::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::v12_1::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::driver::Operation<'a>>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .send(s)
            .await
    }
}

impl<L: AsyncLink> Controller<L, firmware::v12::V12> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::v12::transmission::Sender`].
    pub async fn send<'a, D: Datagram<'a>>(&'a mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::v12::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::v12::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::driver::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::v12::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::driver::Operation<'a>>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .send(s)
            .await
    }
}

impl<L: AsyncLink> Controller<L, firmware::v11::V11> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::v11::transmission::Sender`].
    pub async fn send<'a, D: Datagram<'a>>(&'a mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::v11::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::v11::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::driver::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::v11::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::driver::Operation<'a>>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .send(s)
            .await
    }
}

impl<L: AsyncLink> Controller<L, firmware::v10::V10> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::v10::transmission::Sender`].
    pub async fn send<'a, D: Datagram<'a>>(&'a mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::v10::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::v10::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::driver::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::v10::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::driver::Operation<'a>>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .send(s)
            .await
    }
}

impl<L: AsyncLink> Controller<L, firmware::auto::Auto> {
    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::auto::transmission::Sender`].
    pub async fn send<'a, D: Datagram<'a>>(&'a mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::auto::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::auto::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::driver::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::auto::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::driver::Operation<'a>>::Error>,
    {
        self.sender(self.default_sender_option, FixedSchedule(AsyncSleeper))
            .send(s)
            .await
    }
}

impl<L: AsyncLink, V: Driver> Drop for Controller<L, V> {
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
        datagram::Segment,
        gain::{Gain, GainCalculator, GainCalculatorGenerator, Intensity, Phase, TransducerFilter},
        link::LinkError,
        modulation::{Modulation, ModulationInspectionResult},
    };
    use autd3_driver::{
        autd3_device::AUTD3,
        common::Hz,
        datagram::{GainSTM, ReadsFPGAState},
        firmware::v12_1::V12_1,
    };

    use crate::{
        gain::Uniform,
        link::{Audit, AuditOption, audit::version},
        modulation::{Sine, Static},
    };

    use super::*;

    // GRCOV_EXCL_START
    pub async fn create_controller(
        dev_num: usize,
    ) -> anyhow::Result<Controller<Audit<version::V12_1>, V12_1>> {
        Ok(Controller::open_with(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::<version::V12_1>::new(AuditOption::default()),
        )
        .await?)
    }
    // GRCOV_EXCL_STOP

    #[tokio::test]
    async fn open_failed() {
        assert_eq!(
            Some(AUTDDriverError::Link(LinkError::new("broken"))),
            Controller::<_, V12_1>::open_with(
                [AUTD3::default()],
                Audit::<version::V12_1>::new(AuditOption {
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
        ))
        .await?;

        autd.iter().try_for_each(|dev| {
            assert_eq!(
                *Sine {
                    freq: 150. * Hz,
                    option: Default::default(),
                }
                .calc(&V12_1.firmware_limits())?,
                autd.link[dev.idx()].fpga().modulation_buffer(Segment::S0)
            );
            let f = Uniform {
                intensity: Intensity(0x80),
                phase: Phase::ZERO,
            }
            .init(
                &autd.geometry,
                &autd.environment,
                &TransducerFilter::all_enabled(),
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
                &TransducerFilter::all_enabled(),
            )?
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
        let autd = create_controller(2).await?;

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
            Audit::<version::V12_1>::new(AuditOption::default()),
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
            _ = dev;
        }

        for dev in &autd {
            _ = dev;
        }

        Ok(())
    }
}
