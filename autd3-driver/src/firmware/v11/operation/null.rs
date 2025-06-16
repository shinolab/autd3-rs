pub(crate) use crate::firmware::v10::operation::NullOp;

use super::Operation;

use crate::{firmware::v10::operation::Operation as OperationV10, geometry::Device};

// GRCOV_EXCL_START
impl Operation for NullOp {
    type Error = <Self as OperationV10>::Error;

    fn required_size(&self, device: &Device) -> usize {
        OperationV10::required_size(self, device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        OperationV10::pack(self, device, tx)
    }

    fn is_done(&self) -> bool {
        OperationV10::is_done(self)
    }
}

impl Default for Box<dyn Operation<Error = std::convert::Infallible>> {
    fn default() -> Self {
        Box::new(NullOp)
    }
}
// GRCOV_EXCL_STOP
