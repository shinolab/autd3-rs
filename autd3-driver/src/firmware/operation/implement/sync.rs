use std::{convert::Infallible, mem::size_of};

use crate::{
    datagram::Synchronize,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::geometry::Device;

#[repr(C, align(2))]
struct SyncMsg {
    tag: TypeTag,
    __: u8,
}

pub struct SynchronizeOp {
    is_done: bool,
}

impl SynchronizeOp {
    pub(crate) const fn new() -> Self {
        Self { is_done: false }
    }
}

impl Operation<'_> for SynchronizeOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::operation::write_to_tx(
            tx,
            SyncMsg {
                tag: TypeTag::Sync,
                __: 0,
            },
        );

        self.is_done = true;
        Ok(size_of::<SyncMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<SyncMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl OperationGenerator<'_> for Synchronize {
    type O1 = SynchronizeOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(), Self::O2 {}))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let device = crate::tests::create_device();

        let mut tx = [0x00u8; size_of::<SyncMsg>()];

        let mut op = SynchronizeOp::new();

        assert_eq!(op.required_size(&device), size_of::<SyncMsg>());
        assert!(!op.is_done());
        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done());
        assert_eq!(tx[0], TypeTag::Sync as u8);
    }
}
