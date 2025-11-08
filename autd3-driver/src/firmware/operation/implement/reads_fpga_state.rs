use std::{convert::Infallible, mem::size_of};

use crate::{
    datagram::ReadsFPGAState,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::geometry::Device;

#[repr(C, align(2))]
struct ReadsFPGAStateMsg {
    tag: TypeTag,
    value: bool,
}

pub struct ReadsFPGAStateOp {
    is_done: bool,
    value: bool,
}

impl ReadsFPGAStateOp {
    pub(crate) const fn new(value: bool) -> Self {
        Self {
            is_done: false,
            value,
        }
    }
}

impl Operation<'_> for ReadsFPGAStateOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::operation::write_to_tx(
            tx,
            ReadsFPGAStateMsg {
                tag: TypeTag::ReadsFPGAState,
                value: self.value,
            },
        );

        self.is_done = true;
        Ok(size_of::<ReadsFPGAStateMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<ReadsFPGAStateMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<F: Fn(&Device) -> bool> OperationGenerator<'_> for ReadsFPGAState<F> {
    type O1 = ReadsFPGAStateOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new((self.f)(device)), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case(0x01, true)]
    #[case(0x00, false)]
    fn test(#[case] expected: u8, #[case] value: bool) {
        let device = crate::tests::create_device();

        let mut tx = [0x00u8; 2 * size_of::<ReadsFPGAStateMsg>()];

        let mut op = ReadsFPGAStateOp::new(value);

        assert_eq!(op.required_size(&device), size_of::<ReadsFPGAStateMsg>());
        assert!(!op.is_done());
        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done());
        assert_eq!(tx[0], TypeTag::ReadsFPGAState as u8);
        assert_eq!(tx[1], expected);
    }
}
