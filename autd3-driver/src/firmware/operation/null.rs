use crate::{error::AUTDInternalError, firmware::operation::Operation, geometry::Device};

#[derive(Default)]
pub struct NullOp {}

impl Operation for NullOp {
    // GRCOV_EXCL_START
    fn pack(&mut self, _: &Device, _: &mut [u8]) -> Result<usize, AUTDInternalError> {
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
    #[cfg_attr(miri, ignore)]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let op = NullOp::default();

        assert_eq!(op.required_size(&device), 0);

        assert!(op.is_done());
    }
}
