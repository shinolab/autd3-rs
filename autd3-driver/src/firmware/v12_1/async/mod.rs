pub(crate) mod sender;

use super::V12_1;
use crate::firmware::driver::r#async::{Driver, Sender, TimerStrategy};
use autd3_core::{environment::Environment, link::AsyncLink, sleep::r#async::Sleep};

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T>
    for sender::Sender<'a, L, S, T>
{
    async fn initialize_devices(self) -> Result<(), crate::error::AUTDDriverError> {
        self.inner.initialize_devices().await
    }

    async fn firmware_version(
        self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        self.inner.firmware_version().await
    }

    async fn close(self) -> Result<(), crate::error::AUTDDriverError> {
        self.inner.close().await
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl Driver for V12_1 {
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
        env: &'a Environment,
        option: crate::firmware::driver::SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: autd3_core::link::AsyncLink + 'a,
        S: autd3_core::sleep::r#async::Sleep,
        T: TimerStrategy<S>,
    {
        Self::Sender {
            inner: crate::firmware::v12::r#async::sender::Sender {
                msg_id,
                link,
                geometry,
                sent_flags,
                rx,
                env,
                option,
                timer_strategy,
                _phantom: std::marker::PhantomData,
            },
        }
    }
}
