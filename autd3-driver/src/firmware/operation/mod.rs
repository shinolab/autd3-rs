mod handler;
mod implement;

use crate::geometry::Device;

pub use handler::OperationHandler;
pub use implement::{BoxedDatagram, BoxedOperation, DOperationGenerator, DynOperationGenerator};

#[doc(hidden)]
pub trait Operation<'a>: Send {
    type Error: std::error::Error;

    #[must_use]
    fn required_size(&self, device: &'a Device) -> usize;
    fn pack(&mut self, device: &'a Device, tx: &mut [u8]) -> Result<usize, Self::Error>;
    #[must_use]
    fn is_done(&self) -> bool;
}

#[inline(always)]
pub(crate) fn write_to_tx<T: Sized>(tx: &mut [u8], data: T) {
    unsafe {
        std::ptr::copy_nonoverlapping(&raw const data as _, tx.as_mut_ptr(), size_of::<T>());
    }
}

#[doc(hidden)]
pub trait OperationGenerator<'a> {
    type O1: Operation<'a>;
    type O2: Operation<'a>;

    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)>;
}
