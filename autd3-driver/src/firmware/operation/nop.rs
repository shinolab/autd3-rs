use std::convert::Infallible;

use crate::{
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Nop {
    tag: TypeTag,
    _pad: u8,
}

pub struct NopOp {
    is_done: bool,
}

impl NopOp {
    pub(crate) const fn new() -> Self {
        Self { is_done: false }
    }
}

impl Operation for NopOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::write_to_tx(
            tx,
            Nop {
                tag: TypeTag::Nop,
                _pad: 0,
            },
        );

        self.is_done = true;
        Ok(size_of::<Nop>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<Nop>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::firmware::operation::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn nop_op() {
        let device = create_device(NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<Nop>()];

        let mut op = NopOp::new();

        assert_eq!(op.required_size(&device), size_of::<Nop>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Nop as u8);
    }
}
