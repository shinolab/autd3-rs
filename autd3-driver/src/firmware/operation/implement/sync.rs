use std::convert::Infallible;

use crate::firmware::operation::implement::null::NullOp;
use crate::firmware::operation::{Operation, OperationGenerator};

use crate::{datagram::Synchronize, firmware::tag::TypeTag};

use autd3_core::geometry::Device;

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
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
        Ok(std::mem::size_of::<SyncMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<SyncMsg>()
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
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test() {
        let device = crate::autd3_device::tests::create_device();

        let mut tx = [0x00u8; size_of::<SyncMsg>()];

        let mut op = SynchronizeOp::new();

        assert_eq!(op.required_size(&device), size_of::<SyncMsg>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Sync as u8);
    }
}
