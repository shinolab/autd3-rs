use crate::{
    error::AUTDDriverError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct ReadsFPGAState {
    tag: TypeTag,
    value: bool,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct ReadsFPGAStateOp {
    #[new(default)]
    is_done: bool,
    value: bool,
}

impl Operation for ReadsFPGAStateOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        super::write_to_tx(
            tx,
            ReadsFPGAState {
                tag: TypeTag::ReadsFPGAState,
                value: self.value,
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<ReadsFPGAState>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ReadsFPGAState>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[rstest::rstest]
    #[test]
    #[case(0x01, true)]
    #[case(0x00, false)]
    fn test(#[case] expected: u8, #[case] value: bool) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 2 * size_of::<ReadsFPGAState>()];

        let mut op = ReadsFPGAStateOp::new(value);

        assert_eq!(op.required_size(&device), size_of::<ReadsFPGAState>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ReadsFPGAState as u8);
        assert_eq!(tx[1], expected);
    }
}
