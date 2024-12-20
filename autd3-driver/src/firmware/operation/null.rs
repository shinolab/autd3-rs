use crate::{error::AUTDDriverError, firmware::operation::Operation, geometry::Device};

use derive_new::new;

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct NullOp {
    #[new(default)]
    __: (),
}

impl Operation for NullOp {
    // GRCOV_EXCL_START
    fn pack(&mut self, _: &Device, _: &mut [u8]) -> Result<usize, AUTDDriverError> {
        unreachable!()
    }
    // GRCOV_EXCL_STOP

    fn required_size(&self, _: &Device) -> usize {
        0
    }

    fn is_done(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let op = NullOp::new();

        assert_eq!(op.required_size(&device), 0);

        assert!(op.is_done());
    }
}
