mod sender;

use autd3_core::{
    link::AsyncLink,
    sleep::r#async::{AsyncSleeper, Sleep},
};

use crate::{
    error::AUTDDriverError,
    firmware::{
        auto::Auto,
        driver::{
            FixedSchedule, Version,
            r#async::{Driver, Sender, TimerStrategy},
        },
    },
};

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T>
    for sender::Sender<'a, L, S, T>
{
    async fn initialize_devices(self) -> Result<(), crate::error::AUTDDriverError> {
        match self.version {
            Version::V10 => {
                super::super::v10::V10
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .initialize_devices()
                    .await
            }
            Version::V11 => {
                super::super::v11::V11
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .initialize_devices()
                    .await
            }
            Version::V12 => {
                super::super::v12::V12
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .initialize_devices()
                    .await
            }
        }
    }

    async fn firmware_version(
        self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        match self.version {
            Version::V10 => {
                super::super::v10::V10
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .firmware_version()
                    .await
            }
            Version::V11 => {
                super::super::v11::V11
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .firmware_version()
                    .await
            }
            Version::V12 => {
                super::super::v12::V12
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .firmware_version()
                    .await
            }
        }
    }

    async fn close(self) -> Result<(), crate::error::AUTDDriverError> {
        match self.version {
            Version::V10 => {
                super::super::v10::V10
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .close()
                    .await
            }
            Version::V11 => {
                super::super::v11::V11
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .initialize_devices()
                    .await
            }
            Version::V12 => {
                super::super::v12::V12
                    .sender(
                        self.msg_id,
                        self.link,
                        self.geometry,
                        self.sent_flags,
                        self.rx,
                        self.option,
                        self.timer_strategy,
                    )
                    .initialize_devices()
                    .await
            }
        }
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl Driver for Auto {
    type Sender<'a, L, S, T>
        = sender::Sender<'a, L, S, T>
    where
        L: AsyncLink + 'a,
        S: Sleep,
        T: TimerStrategy<S>;
    type FPGAState = super::super::latest::fpga::FPGAState;

    fn new() -> Self {
        Self {
            version: Version::V12,
        }
    }

    async fn detect_version<'a, L>(
        &mut self,
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::derive::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
    ) -> Result<(), AUTDDriverError>
    where
        L: AsyncLink + 'a,
    {
        let mut sender = self.sender(
            msg_id,
            link,
            geometry,
            sent_flags,
            rx,
            crate::firmware::driver::SenderOption {
                timeout: Some(std::time::Duration::from_secs(1)),
                ..Default::default()
            },
            FixedSchedule(AsyncSleeper),
        );
        let _ = sender
            .send(crate::datagram::ReadsFPGAState::new(|_| false))
            .await;

        let version_list = sender.firmware_version().await?;

        if version_list.is_empty() {
            return Ok(());
        }

        let version = version_list[0];
        if version_list
            .iter()
            .skip(1)
            .any(|v| v.cpu.major != version.cpu.major || v.fpga.major != version.fpga.major)
        {
            return Err(AUTDDriverError::FirmwareVersionMismatch);
        }

        match version.cpu.major.0 {
            0xA2..=0xA2 => {
                self.version = Version::V10;
            }
            0xA3..=0xA3 => {
                self.version = Version::V11;
            }
            0xA4..=0xA4 => {
                self.version = Version::V12;
            }
            _ => {
                return Err(AUTDDriverError::UnsupportedFirmware);
            }
        }

        Ok(())
    }

    fn sender<'a, L, S, T>(
        &self,
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::derive::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
        option: crate::firmware::driver::SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: AsyncLink + 'a,
        S: Sleep,
        T: TimerStrategy<S>,
    {
        Self::Sender {
            msg_id,
            link,
            geometry,
            sent_flags,
            rx,
            option,
            timer_strategy,
            _phantom: std::marker::PhantomData,
            version: self.version,
            limits: self.firmware_limits(),
        }
    }

    fn firmware_limits(&self) -> autd3_core::derive::FirmwareLimits {
        match self.version {
            Version::V10 => super::super::v10::V10.firmware_limits(),
            Version::V11 => super::super::v11::V11.firmware_limits(),
            Version::V12 => super::super::v12::V12.firmware_limits(),
        }
    }
}
