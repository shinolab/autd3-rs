use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    firmware::operation::{Operation, TypeTag},
    geometry::{Device, Geometry},
};

use super::cast;

#[repr(u8)]
pub enum FirmwareVersionType {
    CPUVersionMajor = 0x01,
    CPUVersionMinor = 0x02,
    FPGAVersionMajor = 0x03,
    FPGAVersionMinor = 0x04,
    FPGAFunctions = 0x05,
    Clear = 0x06,
}

#[repr(C, align(2))]
struct FirmInfo {
    tag: TypeTag,
    ty: FirmwareVersionType,
}

#[derive(Default)]
pub struct FirmInfoOp {
    remains: HashMap<usize, usize>,
}

impl Operation for FirmInfoOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<FirmInfo>(tx) = FirmInfo {
            tag: TypeTag::FirmwareVersion,
            ty: match self.remains[&device.idx()] {
                6 => FirmwareVersionType::CPUVersionMajor,
                5 => FirmwareVersionType::CPUVersionMinor,
                4 => FirmwareVersionType::FPGAVersionMajor,
                3 => FirmwareVersionType::FPGAVersionMinor,
                2 => FirmwareVersionType::FPGAFunctions,
                1 => FirmwareVersionType::Clear,
                _ => unreachable!(),
            },
        };
        Ok(std::mem::size_of::<FirmInfo>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<FirmInfo>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains = geometry.devices().map(|device| (device.idx(), 6)).collect();
        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }

    fn commit(&mut self, device: &Device) {
        self.remains.insert(device.idx(), self.remains(device) - 1);
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

        let mut op = FirmInfoOp::default();

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 2));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 6));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 5));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareVersion as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareVersionType::CPUVersionMajor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 4));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareVersion as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareVersionType::CPUVersionMinor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 3));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareVersion as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareVersionType::FPGAVersionMajor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 2));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareVersion as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareVersionType::FPGAVersionMinor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 1));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareVersion as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareVersionType::FPGAFunctions as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareVersion as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareVersionType::Clear as u8);
        });
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);
        let mut tx = [0x00u8; 2 * NUM_DEVICE];

        let mut op = FirmInfoOp::default();

        assert!(op.init(&geometry).is_ok());
        (0..7).for_each(|_| {
            geometry.devices().for_each(|dev| {
                assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
                op.commit(dev);
            });
        });
    }
}
