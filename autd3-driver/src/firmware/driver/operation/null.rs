use super::Operation;
use crate::geometry::Device;

pub struct NullOp;

// GRCOV_EXCL_START
impl Operation<'_> for NullOp {
    type Error = std::convert::Infallible;

    fn required_size(&self, _: &Device) -> usize {
        0
    }

    fn pack(&mut self, _: &Device, _: &mut [u8]) -> Result<usize, Self::Error> {
        unreachable!()
    }

    fn is_done(&self) -> bool {
        true
    }
}

impl Default for Box<dyn Operation<'_, Error = std::convert::Infallible>> {
    fn default() -> Self {
        Box::new(NullOp)
    }
}
// GRCOV_EXCL_STOP
