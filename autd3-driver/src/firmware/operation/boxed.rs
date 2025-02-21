use autd3_core::{datagram::Operation, geometry::Device};

use crate::error::AUTDDriverError;

trait DOperation: Send + Sync {
    #[must_use]
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError>;
    #[must_use]
    fn is_done(&self) -> bool;
}

impl<E, O: Operation<Error = E>> DOperation for O
where
    AUTDDriverError: From<E>,
{
    fn required_size(&self, device: &Device) -> usize {
        O::required_size(self, device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        Ok(O::pack(self, device, tx)?)
    }

    fn is_done(&self) -> bool {
        O::is_done(self)
    }
}

#[doc(hidden)]
pub struct BoxedOperation {
    inner: Box<dyn DOperation>,
}

impl BoxedOperation {
    #[must_use]
    pub fn new<E, O: Operation<Error = E> + 'static>(op: O) -> Self
    where
        AUTDDriverError: From<E>,
    {
        Self {
            inner: Box::new(op),
        }
    }
}

impl Operation for BoxedOperation {
    type Error = AUTDDriverError;

    fn required_size(&self, device: &Device) -> usize {
        self.inner.required_size(device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.pack(device, tx)
    }

    fn is_done(&self) -> bool {
        self.inner.is_done()
    }
}
