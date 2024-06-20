use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::Device,
    get_ultrasound_freq,
};

#[repr(C, align(2))]
struct Sync {
    tag: TypeTag,
    __pad: [u8; 3],
    ecat_sync_base_cnt: u32,
}

#[derive(Default)]
pub struct SyncOp {
    is_done: bool,
}

impl Operation for SyncOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<Sync>(tx) = Sync {
            tag: TypeTag::Sync,
            __pad: [0; 3],
            ecat_sync_base_cnt: get_ultrasound_freq().hz() * 512 / 2000,
        };

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

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<Sync>()];

        let mut op = SyncOp::default();

        assert_eq!(op.required_size(&device), size_of::<Sync>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        let sync_base_cnt = get_ultrasound_freq().hz() * 512 / 2000;
        assert_eq!(tx[0], TypeTag::Sync as u8);
        assert_eq!(tx[4], (sync_base_cnt & 0xFF) as u8);
        assert_eq!(tx[5], ((sync_base_cnt >> 8) & 0xFF) as u8);
        assert_eq!(tx[6], ((sync_base_cnt >> 16) & 0xFF) as u8);
        assert_eq!(tx[7], ((sync_base_cnt >> 24) & 0xFF) as u8);
    }
}
