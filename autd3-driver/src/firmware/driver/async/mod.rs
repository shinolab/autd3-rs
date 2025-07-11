mod strategy;

use super::{FPGAState, SenderOption};
use crate::{error::AUTDDriverError, firmware::version::FirmwareVersion};

use autd3_core::{
    derive::FirmwareLimits,
    environment::Environment,
    geometry::Geometry,
    link::{MsgId, RxMessage},
};

pub use strategy::TimerStrategy;

#[doc(hidden)]
pub trait Sender<'a, L, S, T>: Send {
    fn initialize_devices(self) -> impl std::future::Future<Output = Result<(), AUTDDriverError>>;
    fn firmware_version(
        self,
    ) -> impl std::future::Future<Output = Result<Vec<FirmwareVersion>, AUTDDriverError>>;
    fn close(self) -> impl std::future::Future<Output = Result<(), AUTDDriverError>>;
}

#[doc(hidden)]
pub trait Driver {
    type Sender<'a, L, S, T>: Sender<'a, L, S, T>
    where
        L: autd3_core::link::AsyncLink + 'a,
        S: autd3_core::sleep::r#async::Sleep,
        T: TimerStrategy<S>;
    type FPGAState: FPGAState;

    fn new() -> Self;

    fn detect_version<'a, L>(
        &mut self,
        _msg_id: &'a mut autd3_core::link::MsgId,
        _link: &'a mut L,
        _geometry: &'a autd3_core::derive::Geometry,
        _sent_flags: &'a mut [bool],
        _rx: &'a mut [autd3_core::link::RxMessage],
        _env: &'a Environment,
    ) -> impl std::future::Future<Output = Result<(), AUTDDriverError>>
    where
        L: autd3_core::link::AsyncLink + 'a,
    {
        async { Ok(()) }
    }

    #[allow(clippy::too_many_arguments)]
    fn sender<'a, L, S, T>(
        &self,
        msg_id: &'a mut MsgId,
        link: &'a mut L,
        geometry: &'a Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [RxMessage],
        env: &'a Environment,
        option: SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: autd3_core::link::AsyncLink + 'a,
        S: autd3_core::sleep::r#async::Sleep,
        T: TimerStrategy<S>;
    fn firmware_limits(&self) -> FirmwareLimits;
}
