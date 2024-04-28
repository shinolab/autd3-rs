use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::{Device, Geometry},
};

use super::Remains;

#[repr(C, align(2))]
struct Clear {
    tag: TypeTag,
}

#[derive(Default)]
pub struct ClearOp {
    remains: Remains,
}

impl Operation for ClearOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        cast::<Clear>(tx).tag = TypeTag::Clear;

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<Clear>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<Clear>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::tests::create_geometry;

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 2 * NUM_DEVICE];

        let mut op = ClearOp::default();

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 2));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::Clear as u8);
        });
    }
}
