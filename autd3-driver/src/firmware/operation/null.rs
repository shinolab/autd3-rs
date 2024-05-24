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

    fn init(&mut self, _: &Device) -> Result<(), AUTDInternalError> {
        Ok(())
    }

    fn is_done(&self, _: &Device) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{defined::FREQ_40K, geometry::tests::create_geometry};

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut op = NullOp::default();

        geometry
            .devices()
            .for_each(|dev| assert!(op.init(dev).is_ok()));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 0));

        geometry.devices().for_each(|dev| assert!(op.is_done(dev)));
    }
}
