use crate::{
    error::AUTDInternalError,
    firmware::operation::{write_to_tx, Operation, TypeTag},
    geometry::Device,
};

#[repr(C, align(2))]
struct ForceFan {
    tag: TypeTag,
    value: bool,
}

pub struct ForceFanOp {
    is_done: bool,
    value: bool,
}

impl ForceFanOp {
    pub const fn new(value: bool) -> Self {
        Self {
            is_done: false,
            value,
        }
    }
}

impl Operation for ForceFanOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        unsafe {
            write_to_tx(
                ForceFan {
                    tag: TypeTag::ForceFan,
                    value: self.value,
                },
                tx,
            );
        }

        self.is_done = true;
        Ok(std::mem::size_of::<ForceFan>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ForceFan>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use super::*;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[rstest::rstest]
    #[test]
    #[case(0x01, true)]
    #[cfg_attr(miri, ignore)]
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
