use crate::{
    error::AUTDInternalError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct ForceFan {
    tag: TypeTag,
    value: bool,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct ForceFanOp {
    #[new(default)]
    is_done: bool,
    value: bool,
}

impl Operation for ForceFanOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        tx[..size_of::<ForceFan>()].copy_from_slice(
            ForceFan {
                tag: TypeTag::ForceFan,
                value: self.value,
            }
            .as_bytes(),
        );

        self.is_done = true;
        Ok(size_of::<ForceFan>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<ForceFan>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::*;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[rstest::rstest]
    #[test]
    #[case(0x01, true)]
    #[case(0x00, false)]
    fn test(#[case] expect: u8, #[case] value: bool) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<ForceFan>()];

        let mut op = ForceFanOp::new(value);

        assert_eq!(op.required_size(&device), size_of::<ForceFan>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ForceFan as u8);
        assert_eq!(tx[offset_of!(ForceFan, value)], expect);
    }
}
