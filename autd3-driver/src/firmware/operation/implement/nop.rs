use std::convert::Infallible;

use crate::{
    datagram::Nop,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::geometry::Device;

#[repr(C, align(2))]
struct NopMsg {
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

impl Operation<'_> for NopOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::operation::write_to_tx(
            tx,
            NopMsg {
                tag: TypeTag::Nop,
                _pad: 0,
            },
        );

        self.is_done = true;
        Ok(size_of::<NopMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<NopMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl OperationGenerator<'_> for Nop {
    type O1 = NopOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nop_op() {
        let device = crate::tests::create_device();

        let mut tx = [0x00u8; size_of::<NopMsg>()];

        let mut op = NopOp::new();

        assert_eq!(op.required_size(&device), size_of::<NopMsg>());
        assert!(!op.is_done());
        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done());
        assert_eq!(tx[0], TypeTag::Nop as u8);
    }
}
