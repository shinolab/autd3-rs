pub(crate) mod sender;

use super::V10;
use crate::{
    datagram::{
        Clear, FixedCompletionSteps, Silencer,
        implements::{Null, Static},
    },
    firmware::{
        driver::r#async::{Driver, Sender, TimerStrategy},
        version::FirmwareVersion,
    },
};
use autd3_core::{link::AsyncLink, sleep::r#async::Sleep};

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T>
    for sender::Sender<'a, L, S, T>
{
    async fn initialize_devices(mut self) -> Result<(), crate::error::AUTDDriverError> {
        self.send(crate::datagram::ReadsFPGAState::new(|_| false))
            .await?;

        self.send((
            crate::datagram::Clear::new(),
            crate::datagram::Synchronize::new(),
        ))
        .await
    }

    async fn firmware_version(
        mut self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        use crate::{
            datagram::FirmwareVersionType::*,
            firmware::version::{CPUVersion, FPGAVersion, Major, Minor},
        };

        let cpu_major = self.fetch_firminfo(CPUMajor).await?;
        let cpu_minor = self.fetch_firminfo(CPUMinor).await?;
        let fpga_major = self.fetch_firminfo(FPGAMajor).await?;
        let fpga_minor = self.fetch_firminfo(FPGAMinor).await?;
        let fpga_functions = self.fetch_firminfo(FPGAFunctions).await?;
        self.fetch_firminfo(Clear).await?;

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

    async fn close(mut self) -> Result<(), crate::error::AUTDDriverError> {
        [
            self.send(Silencer {
                config: FixedCompletionSteps {
                    strict_mode: false,
                    ..Default::default()
                },
            })
            .await,
            self.send((Static::default(), Null)).await,
            self.send(Clear {}).await,
            Ok(self.link.close().await?),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl Driver for V10 {
    type Sender<'a, L, S, T>
        = sender::Sender<'a, L, S, T>
    where
        L: autd3_core::link::AsyncLink + 'a,
        S: autd3_core::sleep::r#async::Sleep,
        T: TimerStrategy<S>;
    type FPGAState = super::fpga::FPGAState;

    fn new() -> Self {
        Self
    }

    fn firmware_limits(&self) -> autd3_core::derive::FirmwareLimits {
        <Self as super::super::driver::Driver>::firmware_limits(self)
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
        L: autd3_core::link::AsyncLink + 'a,
        S: autd3_core::sleep::r#async::Sleep,
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
        }
    }
}
