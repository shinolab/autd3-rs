use autd3_core::{link::Link, sleep::Sleep};

use crate::firmware::driver::{Driver, Sender, TimerStrategy};

/// A driver for firmware version 12.1
pub struct V12_1;

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T>
    for super::transmission::Sender<'a, L, S, T>
{
    fn initialize_devices(self) -> Result<(), crate::error::AUTDDriverError> {
        self.inner.initialize_devices()
    }

    fn firmware_version(
        self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        self.inner.firmware_version()
    }

    fn close(self) -> Result<(), crate::error::AUTDDriverError> {
        self.inner.close()
    }
}

impl Driver for V12_1 {
    type Sender<'a, L, S, T>
        = super::transmission::Sender<'a, L, S, T>
    where
        L: autd3_core::link::Link + 'a,
        S: autd3_core::sleep::Sleep,
        T: TimerStrategy<S>;
    type FPGAState = super::fpga::FPGAState;

    fn new() -> Self {
        Self
    }

    fn firmware_limits(&self) -> autd3_core::derive::FirmwareLimits {
        crate::firmware::v12::V12.firmware_limits()
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
        L: autd3_core::link::Link + 'a,
        S: autd3_core::sleep::Sleep,
        T: TimerStrategy<S>,
    {
        Self::Sender {
            inner: crate::firmware::v12::transmission::Sender {
                msg_id,
                link,
                geometry,
                sent_flags,
                rx,
                option,
                timer_strategy,
                _phantom: std::marker::PhantomData,
            },
        }
    }
}
