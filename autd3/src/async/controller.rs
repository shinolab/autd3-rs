use autd3_core::{
    datagram::{Datagram, DeviceMask, Inspectable, InspectionResult},
    derive::{Device, Geometry},
    environment::Environment,
    link::{Ack, AsyncLink, MsgId, RxMessage},
    sleep::r#async::Sleep,
};

use autd3_driver::{
    error::AUTDDriverError,
    firmware::{
        r#async::Sender, fpga::FPGAState, transmission::SenderOption, version::FirmwareVersion,
    },
};

pub use autd3_core::sleep::r#async::AsyncSleeper;

/// An asynchronous controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
pub struct Controller<L: AsyncLink> {
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

impl<L: AsyncLink> std::ops::Deref for Controller<L> {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl<L: AsyncLink> std::ops::DerefMut for Controller<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geometry
    }
}

impl<L: AsyncLink> Controller<L> {
    /// Equivalent to [`Self::open_with`] with default [`SenderOption`] and [`AsyncSleeper`].
    pub async fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        link: L,
    ) -> Result<Self, AUTDDriverError> {
        Self::open_with(devices, link, Default::default(), AsyncSleeper).await
    }

    /// Opens a controller with a [`SenderOption`].
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub async fn open_with<D: Into<Device>, F: IntoIterator<Item = D>, S: Sleep>(
        devices: F,
        mut link: L,
        option: SenderOption,
        sleeper: S,
    ) -> Result<Self, AUTDDriverError> {
        let geometry = Geometry::new(devices.into_iter().map(|d| d.into()).collect());
        let environment = Environment::default();

        link.open(&geometry).await?;

        let mut cnt = Controller {
            link,
            msg_id: MsgId::new(0),
            sent_flags: vec![false; geometry.len()],
            rx_buf: vec![RxMessage::new(0, Ack::new(0x00, 0x00)); geometry.len()],
            geometry,
            environment,
            default_sender_option: option,
        };

        cnt.sender(option, sleeper).initialize_devices().await?;

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
    pub fn sender<S: Sleep>(&mut self, option: SenderOption, sleeper: S) -> Sender<'_, L, S> {
        Sender::new(
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

    /// Sends a data to the devices. This is a shortcut for [`autd3_driver::firmware::transmission::Sender`].
    pub async fn send<'a, D: Datagram<'a>>(&'a mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::operation::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::operation::Operation<'a>>::Error>,
    {
        self.sender(self.default_sender_option, AsyncSleeper)
            .send(s)
            .await
    }

    /// Returns the inspection result.
    pub fn inspect<'a, I: Inspectable<'a>>(
        &'a self,
        s: I,
    ) -> Result<InspectionResult<I::Result>, I::Error> {
        s.inspect(&self.geometry, &self.environment, &DeviceMask::AllEnabled)
    }

    /// Closes the controller.
    pub async fn close(mut self) -> Result<(), AUTDDriverError> {
        self.close_impl(self.default_sender_option, AsyncSleeper)
            .await
    }

    /// Returns the firmware version of the devices.
    pub async fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDDriverError> {
        self.sender(self.default_sender_option, AsyncSleeper)
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
    async fn close_impl<S: Sleep>(
        &mut self,
        option: SenderOption,
        sleeper: S,
    ) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Ok(());
        }
        self.sender(option, sleeper).close().await
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
                        .close_impl(self.default_sender_option, AsyncSleeper)
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
        firmware::{Intensity, Phase, Segment},
        gain::{Gain, GainCalculator, GainCalculatorGenerator, TransducerMask},
        link::LinkError,
        modulation::{Modulation, ModulationInspectionResult},
    };
    use autd3_driver::{
        autd3_device::AUTD3,
        common::Hz,
        datagram::{GainSTM, ReadsFPGAState},
    };

    use crate::{
        gain::Uniform,
        link::{Audit, AuditOption},
        modulation::{Sine, Static},
    };

    use super::*;

    pub async fn create_controller(dev_num: usize) -> Result<Controller<Audit>, AUTDDriverError> {
        Controller::open(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::new(AuditOption::default()),
        )
        .await
    }

    #[tokio::test]
    async fn deref_mut() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = create_controller(1).await?;
        assert_eq!(1, autd.len());
        autd.reconfigure(|dev| dev);
        Ok(())
    }

    #[tokio::test]
    async fn geometry() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = create_controller(1).await?;
        assert_eq!(1, autd.geometry().len());
        autd.geometry_mut().reconfigure(|dev| dev);
        Ok(())
    }

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
    async fn send() -> Result<(), Box<dyn std::error::Error>> {
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

        autd.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn inspect() -> Result<(), Box<dyn std::error::Error>> {
        let autd = create_controller(2).await?;

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

        autd.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn firmware_version() -> Result<(), Box<dyn std::error::Error>> {
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
    async fn firmware_version_err() -> Result<(), Box<dyn std::error::Error>> {
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
    async fn close() -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut autd = create_controller(1).await?;
            autd.close_impl(SenderOption::default(), AsyncSleeper)
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
    async fn fpga_state() -> Result<(), Box<dyn std::error::Error>> {
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
            assert!(states[0].is_some_and(|s| s.is_thermal_assert()));
            assert!(states[1].is_some_and(|s| !s.is_thermal_assert()));
        }

        {
            autd.link_mut()[0].fpga_mut().deassert_thermal_sensor();
            autd.link_mut()[1].fpga_mut().assert_thermal_sensor();

            let states = autd.fpga_state().await?;
            assert_eq!(2, states.len());
            assert!(states[0].is_some_and(|s| !s.is_thermal_assert()));
            assert!(states[1].is_some_and(|s| s.is_thermal_assert()));
        }

        autd.send(ReadsFPGAState::new(|dev| dev.idx() == 1)).await?;
        {
            let states = autd.fpga_state().await?;
            assert_eq!(2, states.len());
            assert!(states[0].is_none());
            assert!(states[1].is_some_and(|s| s.is_thermal_assert()));
        }

        Ok(())
    }

    #[tokio::test]
    async fn into_iter() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = create_controller(1).await?;
        (&mut autd).into_iter().for_each(|dev| {
            _ = dev;
        });
        (&autd).into_iter().for_each(|dev| {
            _ = dev;
        });
        Ok(())
    }
}
