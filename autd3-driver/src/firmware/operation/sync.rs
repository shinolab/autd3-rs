use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::{Device, Geometry},
};

use super::Remains;

#[repr(C, align(2))]
struct Sync {
    tag: TypeTag,
    __pad: [u8; 3],
    ecat_sync_base_cnt: u32,
}

#[derive(Default)]
pub struct SyncOp {
    remains: Remains,
}

impl Operation for SyncOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert!(tx.len() >= std::mem::size_of::<Sync>());

        *cast::<Sync>(tx) = Sync {
            tag: TypeTag::Sync,
            __pad: [0; 3],
            ecat_sync_base_cnt: device.ultrasound_freq() * 512 / 2000,
        };

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<Sync>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<Sync>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, |_| 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{defined::FREQ_40K, geometry::tests::create_geometry};

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut tx = [0x00u8; 8 * NUM_DEVICE];

        let mut op = SyncOp::default();

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 8));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 8..]).is_ok());
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            let sync_base_cnt = dev.ultrasound_freq() * 512 / 2000;
            assert_eq!(tx[dev.idx() * 8], TypeTag::Sync as u8);
            assert_eq!(tx[dev.idx() * 8 + 4], (sync_base_cnt & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * 8 + 5], ((sync_base_cnt >> 8) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * 8 + 6], ((sync_base_cnt >> 16) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * 8 + 7], ((sync_base_cnt >> 24) & 0xFF) as u8);
        });
    }
}
