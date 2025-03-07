use crate::geometry::Device;

#[doc(hidden)]
pub trait Operation: Send + Sync {
    type Error: std::error::Error;

    #[must_use]
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error>;
    #[must_use]
    fn is_done(&self) -> bool;
}

#[doc(hidden)]
pub struct NullOp;

// GRCOV_EXCL_START
impl Operation for NullOp {
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

impl Default for Box<dyn Operation<Error = std::convert::Infallible>> {
    fn default() -> Self {
        Box::new(NullOp)
    }
}
// GRCOV_EXCL_STOP
