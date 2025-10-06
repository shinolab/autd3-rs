mod handler;
mod implement;

use crate::geometry::Device;

use zerocopy::{Immutable, IntoBytes};

pub use handler::OperationHandler;
pub use implement::{BoxedDatagram, BoxedOperation, DOperationGenerator, DynOperationGenerator};

#[doc(hidden)]
pub trait Operation<'a>: Send + Sync {
    type Error: std::error::Error;

    #[must_use]
    fn required_size(&self, device: &'a Device) -> usize;
    fn pack(&mut self, device: &'a Device, tx: &mut [u8]) -> Result<usize, Self::Error>;
    #[must_use]
    fn is_done(&self) -> bool;
}

#[inline(always)]
pub(crate) fn write_to_tx<T: IntoBytes + Immutable>(tx: &mut [u8], data: T) {
    tx[..size_of::<T>()].copy_from_slice(data.as_bytes());
}

#[doc(hidden)]
pub trait OperationGenerator<'a> {
    type O1: Operation<'a>;
    type O2: Operation<'a>;

    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)>;
}
