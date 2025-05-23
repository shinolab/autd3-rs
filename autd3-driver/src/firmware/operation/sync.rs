use std::convert::Infallible;

use crate::{
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Sync {
    tag: TypeTag,
    __: u8,
}

pub struct SyncOp {
    is_done: bool,
}

impl SyncOp {
    pub(crate) const fn new() -> Self {
        Self { is_done: false }
    }
}

impl Operation for SyncOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::write_to_tx(
            tx,
            Sync {
                tag: TypeTag::Sync,
                __: 0,
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<Sync>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<Sync>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {

    use std::mem::size_of;

    use super::*;
    use crate::firmware::operation::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn test() {
        let device = create_device(NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<Sync>()];

        let mut op = SyncOp::new();

        assert_eq!(op.required_size(&device), size_of::<Sync>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Sync as u8);
    }
}
