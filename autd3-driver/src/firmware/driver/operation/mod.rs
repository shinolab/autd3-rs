pub(crate) mod boxed;
mod handler;
mod null;
mod version;

pub use boxed::{BoxedDatagram, BoxedOperation, DOperationGenerator, DynOperationGenerator};
pub use handler::OperationHandler;
pub use null::NullOp;
pub use version::Version;

use crate::geometry::Device;

use zerocopy::{Immutable, IntoBytes};

#[doc(hidden)]
pub trait Operation<'dev>: Send + Sync {
    type Error: std::error::Error;

    #[must_use]
    fn required_size(&self, device: &'dev Device) -> usize;
    fn pack(&mut self, device: &'dev Device, tx: &mut [u8]) -> Result<usize, Self::Error>;
    #[must_use]
    fn is_done(&self) -> bool;
}

#[inline(always)]
pub(crate) fn write_to_tx<T: IntoBytes + Immutable>(tx: &mut [u8], data: T) {
    tx[..size_of::<T>()].copy_from_slice(data.as_bytes());
}
