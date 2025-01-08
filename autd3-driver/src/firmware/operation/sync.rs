use crate::{
    defined::ultrasound_freq,
    error::AUTDDriverError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Sync {
    tag: TypeTag,
    __: u8,
    ufreq_mult: u16,
    base_cnt: u16,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct SyncOp {
    #[new(default)]
    is_done: bool,
}

impl Operation for SyncOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        let ultrasound_freq = ultrasound_freq().hz();
        let mult = ultrasound_freq / 125;
        let base_cnt = (ultrasound_freq as u64 * 256 * 500) / 1000000;

        super::write_to_tx(
            tx,
            Sync {
                tag: TypeTag::Sync,
                __: 0,
                ufreq_mult: mult as _,
                base_cnt: base_cnt as _,
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
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<Sync>()];

        let mut op = SyncOp::new();

        assert_eq!(op.required_size(&device), size_of::<Sync>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Sync as u8);
    }
}
