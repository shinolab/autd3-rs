use crate::{
    error::AUTDInternalError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Clear {
    tag: TypeTag,
    __: u8,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct ClearOp {
    #[new(default)]
    is_done: bool,
}

impl Operation for ClearOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        tx[..size_of::<Clear>()].copy_from_slice(
            Clear {
                tag: TypeTag::Clear,
                __: 0,
            }
            .as_bytes(),
        );

        self.is_done = true;
        Ok(std::mem::size_of::<Clear>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<Clear>()
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

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<Clear>()];

        let mut op = ClearOp::new();

        assert_eq!(op.required_size(&device), size_of::<Clear>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Clear as u8);
    }
}
