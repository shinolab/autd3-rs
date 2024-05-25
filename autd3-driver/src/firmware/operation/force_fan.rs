use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
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
    pub fn new(value: bool) -> Self {
        Self {
            is_done: false,
            value,
        }
    }
}

impl Operation for ForceFanOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<ForceFan>(tx) = ForceFan {
            tag: TypeTag::ForceFan,
            value: self.value,
        };

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
