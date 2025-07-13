use std::convert::Infallible;

use super::OperationGenerator;
use crate::{
    datagram::Clear,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
};

use autd3_core::geometry::Device;

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct ClearMsg {
    tag: TypeTag,
    __: u8,
}

pub struct ClearOp {
    is_done: bool,
}

impl ClearOp {
    pub(crate) const fn new() -> Self {
        Self { is_done: false }
    }
}

impl Operation<'_> for ClearOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            ClearMsg {
                tag: TypeTag::Clear,
                __: 0,
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<ClearMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ClearMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl OperationGenerator<'_> for Clear {
    type O1 = ClearOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test() {
        let device = crate::autd3_device::tests::create_device();

        let mut tx = [0x00u8; size_of::<ClearMsg>()];

        let mut op = ClearOp::new();

        assert_eq!(op.required_size(&device), size_of::<ClearMsg>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Clear as u8);
    }
}
