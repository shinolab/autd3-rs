/// Asynchronous module.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub mod r#async;

pub(crate) mod operation;
mod option;
mod parallel_mode;
mod strategy;

pub(crate) use operation::{
    BoxedOperation, DOperationGenerator, DynOperationGenerator, NullOp, write_to_tx,
};

pub use operation::{BoxedDatagram, Operation, OperationHandler, Version};
pub use option::SenderOption;
pub use parallel_mode::ParallelMode;
pub use strategy::{FixedDelay, FixedSchedule, TimerStrategy};

use autd3_core::{
    derive::FirmwareLimits,
    geometry::Geometry,
    link::{MsgId, RxMessage},
};

use crate::{error::AUTDDriverError, firmware::version::FirmwareVersion};

#[doc(hidden)]
pub trait Sender<'a, L, S, T> {
    fn initialize_devices(self) -> Result<(), AUTDDriverError>;
    fn firmware_version(self) -> Result<Vec<FirmwareVersion>, AUTDDriverError>;
    fn close(self) -> Result<(), AUTDDriverError>;
}

#[doc(hidden)]
pub trait FPGAState
where
    Self: Sized,
{
    fn from_rx(rx: &RxMessage) -> Option<Self>;
}

#[doc(hidden)]
pub trait Driver {
    type Sender<'a, L, S, T>: Sender<'a, L, S, T>
    where
        L: autd3_core::link::Link + 'a,
        S: autd3_core::sleep::Sleep,
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
    ) -> Result<(), AUTDDriverError>
    where
        L: autd3_core::link::Link + 'a,
    {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn sender<'a, L, S, T>(
        &self,
        msg_id: &'a mut MsgId,
        link: &'a mut L,
        geometry: &'a Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [RxMessage],
        option: SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: autd3_core::link::Link + 'a,
        S: autd3_core::sleep::Sleep,
        T: TimerStrategy<S>;
    fn firmware_limits(&self) -> FirmwareLimits;
}
