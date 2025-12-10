use autd3_core::{
    datagram::{Datagram, DeviceMask},
    datagram::{Inspectable, InspectionResult},
    environment::Environment,
    link::{Ack, AsyncLink, MsgId, RxMessage},
    sleep::AsyncSleeper,
};

use autd3_driver::{
    error::AUTDDriverError,
    firmware::{fpga::FPGAState, transmission::SenderOption, version::FirmwareVersion},
    geometry::{Device, Geometry},
};

/// A async controller for the AUTD devices.
///
/// All operations to the devices are done through this struct.
pub struct AsyncController<L: AsyncLink> {
    link: L,
    geometry: Geometry,
    /// THe environment where the devices are placed.
    pub environment: Environment,
    msg_id: MsgId,
    sent_flags: Vec<bool>,
    rx_buf: Vec<RxMessage>,
    /// The default sender option used for [`send`](AsyncController::send).
    pub default_sender_option: SenderOption,
}

/// A struct to send the [`Datagram`] to the devices.
pub struct AsyncSender<'a, L: AsyncLink, S: AsyncSleeper> {
    inner: autd3_driver::firmware::transmission::AsyncSender<'a, L, S>,
}

impl<'a, L: AsyncLink, S: AsyncSleeper> AsyncSender<'a, L, S> {
    /// Send the [`Datagram`] to the devices.
    pub async fn send<D: Datagram<'a>>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::operation::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::operation::Operation<'a>>::Error>,
    {
        self.inner.send(s).await
    }
}

impl<L: AsyncLink> std::ops::Deref for AsyncController<L> {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl<L: AsyncLink> std::ops::DerefMut for AsyncController<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geometry
    }
}

impl<L: AsyncLink> AsyncController<L> {
    /// Opens a controller with a [`SenderOption`].
    ///
    /// Opens link, and then initialize and synchronize the devices. The `timeout` is used to send data for initialization and synchronization.
    pub async fn open_with<D: Into<Device>, F: IntoIterator<Item = D>, S: AsyncSleeper>(
        devices: F,
        mut link: L,
        option: SenderOption,
        sleeper: S,
    ) -> Result<Self, AUTDDriverError> {
        let geometry = Geometry::new(devices.into_iter().map(|d| d.into()).collect());
        let environment = Environment::default();

        link.open(&geometry).await?;

        let mut cnt = AsyncController {
            link,
            msg_id: MsgId::new(0),
            sent_flags: vec![false; geometry.len()],
            rx_buf: vec![RxMessage::new(0, Ack::new(0x00, 0x00)); geometry.len()],
            geometry,
            environment,
            default_sender_option: option,
        };

        cnt.raw_sender(option, sleeper).initialize_devices().await?;

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

    /// Returns the [`AsyncSender`] to send data to the devices with the given [`AsyncSleeper`].
    pub fn sender_with_sleeper<S: AsyncSleeper>(
        &mut self,
        option: SenderOption,
        sleeper: S,
    ) -> AsyncSender<'_, L, S> {
        AsyncSender {
            inner: self.raw_sender(option, sleeper),
        }
    }

    /// Sends a data to the devices. This is a shortcut for [`AsyncSender::send`].
    ///
    /// [`AsyncSender::send`]: autd3_driver::firmware::transmission::AsyncSender::send
    pub async fn send<'a, D: Datagram<'a>>(&'a mut self, s: D, sleeper: impl AsyncSleeper) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: autd3_driver::firmware::operation::OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O1 as autd3_driver::firmware::operation::Operation<'a>>::Error>
            + From<<<D::G as autd3_driver::firmware::operation::OperationGenerator<'a>>::O2 as autd3_driver::firmware::operation::Operation<'a>>::Error>,
    {
        self.sender_with_sleeper(self.default_sender_option, sleeper)
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
    pub async fn close(mut self, sleeper: impl AsyncSleeper) -> Result<(), AUTDDriverError> {
        self.close_impl(self.default_sender_option, sleeper).await
    }

    /// Returns the firmware version of the devices.
    pub async fn firmware_version(
        &mut self,
        sleeper: impl AsyncSleeper,
    ) -> Result<Vec<FirmwareVersion>, AUTDDriverError> {
        self.raw_sender(self.default_sender_option, sleeper)
            .firmware_version()
            .await
    }

    /// Returns the FPGA state of the devices.
    ///
    /// To get the state of devices, enable reads FPGA state mode by [`ReadsFPGAState`] before calling this method.
    /// The returned value is [`None`] if the reads FPGA state mode is disabled for the device.
    ///
    /// [`ReadsFPGAState`]: autd3_driver::datagram::ReadsFPGAState
    pub async fn fpga_state(&mut self) -> Result<Vec<Option<FPGAState>>, AUTDDriverError> {
        self.link.ensure_is_open()?;
        self.link.receive(&mut self.rx_buf).await?;
        Ok(self.rx_buf.iter().map(FPGAState::from_rx).collect())
    }
}

impl<L: AsyncLink> AsyncController<L> {
    fn raw_sender<S: AsyncSleeper>(
        &mut self,
        option: SenderOption,
        sleeper: S,
    ) -> autd3_driver::firmware::transmission::AsyncSender<'_, L, S> {
        autd3_driver::firmware::transmission::AsyncSender::new(
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

    async fn close_impl(
        &mut self,
        option: SenderOption,
        sleeper: impl AsyncSleeper,
    ) -> Result<(), AUTDDriverError> {
        if !self.link.is_open() {
            return Ok(());
        }
        self.raw_sender(option, sleeper).close().await
    }
}

impl<'a, L: AsyncLink> IntoIterator for &'a AsyncController<L> {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter()
    }
}

impl<'a, L: AsyncLink> IntoIterator for &'a mut AsyncController<L> {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.geometry.iter_mut()
    }
}
