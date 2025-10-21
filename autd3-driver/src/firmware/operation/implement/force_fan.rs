use std::convert::Infallible;

use crate::firmware::operation::implement::null::NullOp;
use crate::firmware::operation::{Operation, OperationGenerator};
use crate::{datagram::ForceFan, firmware::tag::TypeTag};

use autd3_core::geometry::Device;

#[repr(C, align(2))]
struct ForceFanMsg {
    tag: TypeTag,
    value: bool,
}

pub struct ForceFanOp {
    is_done: bool,
    value: bool,
}

impl ForceFanOp {
    pub(crate) const fn new(value: bool) -> Self {
        Self {
            is_done: false,
            value,
        }
    }
}

impl Operation<'_> for ForceFanOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::operation::write_to_tx(
            tx,
            ForceFanMsg {
                tag: TypeTag::ForceFan,
                value: self.value,
            },
        );

        self.is_done = true;
        Ok(size_of::<ForceFanMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<ForceFanMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<F: Fn(&Device) -> bool> OperationGenerator<'_> for ForceFan<F> {
    type O1 = ForceFanOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new((self.f)(device)), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::*;

    #[rstest::rstest]
    #[case(0x01, true)]
    #[case(0x00, false)]
    fn test(#[case] expect: u8, #[case] value: bool) {
        let device = crate::tests::create_device();

        let mut tx = [0x00u8; size_of::<ForceFanMsg>()];

        let mut op = ForceFanOp::new(value);

        assert_eq!(op.required_size(&device), size_of::<ForceFanMsg>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ForceFan as u8);
        assert_eq!(tx[offset_of!(ForceFanMsg, value)], expect);
    }
}
