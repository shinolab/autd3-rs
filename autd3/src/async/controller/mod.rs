mod group;
mod sender;

use crate::{controller::SenderOption, error::AUTDError, gain::Null, prelude::Static};

use std::time::Duration;

use autd3_core::{
    defined::DEFAULT_TIMEOUT,
    geometry::IntoDevice,
    link::{AsyncLink, AsyncLinkBuilder},
};

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

pub use group::Group;
pub use sender::AsyncSleeper;

use derive_more::{Deref, DerefMut};
use getset::{Getters, MutGetters};
use sender::{sleep::AsyncSleep, Sender};
use tracing;
use zerocopy::FromZeros;

/// A controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
#[derive(Deref, DerefMut, Getters, MutGetters)]
pub struct Controller<L: AsyncLink> {
    #[getset(get = "pub", get_mut = "pub")]
    link: L,
    #[getset(get = "pub", get_mut = "pub")]
    #[deref]
    #[deref_mut]
    geometry: Geometry,
    tx_buf: Vec<TxMessage>,
    rx_buf: Vec<RxMessage>,
}

impl<L: AsyncLink> Controller<L> {
    /// Equivalent to [`Self::open_with_timeout`] with a timeout of [`DEFAULT_TIMEOUT`].
    pub async fn open<D: IntoDevice, F: IntoIterator<Item = D>, B: AsyncLinkBuilder<L = L>>(
        devices: F,
        link_builder: B,
    ) -> Result<Controller<B::L>, AUTDError> {
        Self::open_with_timeout(devices, link_builder, DEFAULT_TIMEOUT).await
    }

    /// Opens a controller with a timeout.
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub async fn open_with_timeout<
        D: IntoDevice,
        F: IntoIterator<Item = D>,
        B: AsyncLinkBuilder<L = L>,
    >(
        devices: F,
        link_builder: B,
        timeout: Duration,
    ) -> Result<Self, AUTDError> {
        tracing::debug!("Opening a controller with timeout {:?})", timeout);

        let devices = devices
            .into_iter()
            .enumerate()
            .map(|(i, d)| d.into_device(i as _))
            .collect();

        let geometry = Geometry::new(devices);
        Controller {
            link: link_builder.open(&geometry).await?,
            tx_buf: vec![TxMessage::new_zeroed(); geometry.len()], // Do not use `num_devices` here because the devices may be disabled.
            rx_buf: vec![RxMessage::new(0, 0); geometry.len()],
            geometry,
        }
        .open_impl(timeout)
        .await
    }

    pub fn sender<'a, S: AsyncSleep>(
        &'a mut self,
        sleeper: S,
        option: SenderOption,
    ) -> Sender<'a, L, S> {
        Sender {
            link: &mut self.link,
            geometry: &mut self.geometry,
            tx: &mut self.tx_buf,
            rx: &mut self.rx_buf,
            sleeper,
            option,
        }
    }

    /// Sends a data to the devices.
    ///
    /// If the [`Datagram::timeout`] value is
    /// - greater than 0, this function waits until the sent data is processed by the device or the specified timeout time elapses. If it cannot be confirmed that the sent data has been processed by the device, [`AUTDDriverError::ConfirmResponseFailed`] is returned.
    /// - 0, the `send` function does not check whether the sent data has been processed by the device.
    ///
    /// The calculation of each [`Datagram`] is executed in parallel for each device if the number of enabled devices is greater than the [`Datagram::parallel_threshold`].
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        self.sender(AsyncSleeper::default(), SenderOption::default())
            .send(s)
            .await
    }

    pub(crate) async fn open_impl(mut self, timeout: Duration) -> Result<Self, AUTDError> {
        let timeout = Some(timeout);

        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data (here, we use `ForceFan` because it is the lightest).
        let _ = self.send(ForceFan::new(|_| false)).await;

        let mut sender = self.sender(
            AsyncSleeper::default(),
            SenderOption {
                timeout,
                ..Default::default()
            },
        );

        #[cfg(feature = "dynamic_freq")]
        {
            tracing::debug!(
                "Configuring ultrasound frequency to {:?}",
                autd3_driver::defined::ultrasound_freq()
            );
            sender
                .send(autd3_driver::datagram::ConfigureFPGAClock::new())
                .await?;
        }

        sender.send((Clear::new(), Synchronize::new())).await?;
        Ok(self)
    }

    async fn close_impl(&mut self) -> Result<(), AUTDDriverError> {
        tracing::info!("Closing controller");

        if !self.link.is_open() {
            tracing::warn!("Link is already closed");
            return Ok(());
        }

        self.geometry.iter_mut().for_each(|dev| dev.enable = true);
        [
            self.send(Silencer {
                config: FixedCompletionSteps {
                    strict_mode: false,
                    ..Default::default()
                },
                target: autd3_driver::firmware::fpga::SilencerTarget::Intensity,
            })
            .await,
            self.send((Static::default(), Null::default())).await,
            self.send(Clear {}).await,
            Ok(self.link.close().await?),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }

    /// Closes the controller.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn close(mut self) -> Result<(), AUTDDriverError> {
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

    /// Returns  the firmware version of the devices.
    pub async fn firmware_version(&mut self) -> Result<Vec<FirmwareVersion>, AUTDError> {
        use autd3_driver::firmware::version::{CPUVersion, FPGAVersion, Major, Minor};
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
    /// let mut autd = Controller::builder([AUTD3::default()]).open(Nop::builder())?;
    ///
    /// autd.send(ReadsFPGAState::new(|_| true))?;
    ///
    /// let states = autd.fpga_state()?;
    /// Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadsFPGAState`]: autd3_driver::datagram::ReadsFPGAState
    pub async fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDError> {
        if !self.link.is_open() {
            return Err(AUTDError::Driver(
                autd3_driver::error::AUTDDriverError::LinkClosed,
            ));
        }
        if self.link.receive(&mut self.rx_buf).await? {
            Ok(self.rx_buf.iter().map(FPGAState::from_rx).collect())
        } else {
            Err(AUTDError::ReadFPGAStateFailed)
        }
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
    ///
    pub unsafe fn from_boxed_link(cnt: Controller<Box<dyn AsyncLink>>) -> Controller<L> {
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

impl<L: AsyncLink> Drop for Controller<L> {
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
    use autd3_core::{
        defined::mm,
        derive::{DatagramOption, Modulation, Segment},
        gain::{Drive, EmitIntensity, Gain, GainContext, GainContextGenerator, Phase},
        link::LinkError,
    };
    use autd3_driver::{
        autd3_device::AUTD3,
        datagram::{GainSTM, ReadsFPGAState},
        defined::Hz,
    };

    use crate::{
        gain::Uniform,
        link::{Audit, AuditOption},
        prelude::Sine,
    };

    use super::*;

    // GRCOV_EXCL_START
    pub async fn create_controller(dev_num: usize) -> anyhow::Result<Controller<Audit>> {
        Ok(Controller::open(
            (0..dev_num).map(|_| AUTD3::default()),
            Audit::builder(AuditOption::default()),
        )
        .await?)
    }
    // GRCOV_EXCL_STOP

    #[tokio::test]
    async fn open_failed() {
        assert_eq!(
            Some(AUTDError::Driver(AUTDDriverError::SendDataFailed)),
            Controller::open(
                [AUTD3::default()],
                Audit::builder(AuditOption {
                    down: true,
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
                        drive: Drive {
                            intensity: EmitIntensity(0x80),
                            phase: Phase::ZERO,
                        },
                    },
                    Uniform {
                        drive: Drive {
                            intensity: EmitIntensity(0x81),
                            phase: Phase::ZERO,
                        },
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
                drive: Drive {
                    intensity: EmitIntensity(0x80),
                    phase: Phase::ZERO,
                },
            }
            .init(&autd.geometry, None, &DatagramOption::default())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                drive: Drive {
                    intensity: EmitIntensity(0x81),
                    phase: Phase::ZERO,
                },
            }
            .init(&autd.geometry, None, &DatagramOption::default())?
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
            Err(AUTDError::ReadFirmwareVersionFailed(vec![false, false])),
            autd.firmware_version().await
        );
        Ok(())
    }

    #[tokio::test]
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
                Err(AUTDDriverError::Link(LinkError::new("broken".to_owned()))),
                autd.close().await
            );
        }

        {
            let mut autd = create_controller(1).await?;
            autd.link_mut().down();
            assert_eq!(Err(AUTDDriverError::SendDataFailed), autd.close().await);
        }

        Ok(())
    }

    #[tokio::test]
    async fn fpga_state() -> anyhow::Result<()> {
        let mut autd = Controller::open(
            [AUTD3::default(), AUTD3::default()],
            Audit::builder(AuditOption::default()),
        )
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
                        drive: Drive {
                            intensity: EmitIntensity(0x80),
                            phase: Phase::ZERO,
                        },
                    },
                    Uniform {
                        drive: Drive {
                            intensity: EmitIntensity(0x81),
                            phase: Phase::ZERO,
                        },
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
                drive: Drive {
                    intensity: EmitIntensity(0x80),
                    phase: Phase::ZERO,
                },
            }
            .init(&autd.geometry, None, &DatagramOption::default())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link[dev.idx()].fpga().drives_at(Segment::S0, 0)
            );
            let f = Uniform {
                drive: Drive {
                    intensity: EmitIntensity(0x81),
                    phase: Phase::ZERO,
                },
            }
            .init(&autd.geometry, None, &DatagramOption::default())?
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
