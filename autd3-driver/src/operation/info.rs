use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    geometry::{Device, Geometry},
    operation::{Operation, TypeTag},
};

use super::cast;

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum FirmwareInfoType {
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
    ty: FirmwareInfoType,
}

#[derive(Default)]
pub struct FirmInfoOp {
    remains: HashMap<usize, usize>,
}

impl Operation for FirmInfoOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let d = cast::<FirmInfo>(tx);
        d.tag = TypeTag::FirmwareInfo;
        d.ty = match self.remains[&device.idx()] {
            6 => FirmwareInfoType::CPUVersionMajor,
            5 => FirmwareInfoType::CPUVersionMinor,
            4 => FirmwareInfoType::FPGAVersionMajor,
            3 => FirmwareInfoType::FPGAVersionMinor,
            2 => FirmwareInfoType::FPGAFunctions,
            1 => FirmwareInfoType::Clear,
            _ => unreachable!(),
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
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareInfo as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareInfoType::CPUVersionMajor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 4));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareInfo as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareInfoType::CPUVersionMinor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 3));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareInfo as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareInfoType::FPGAVersionMajor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 2));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareInfo as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareInfoType::FPGAVersionMinor as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 1));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareInfo as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareInfoType::FPGAFunctions as u8);
        });

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));
        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::FirmwareInfo as u8);
            let flag = tx[dev.idx() * 2 + 1];
            assert_eq!(flag, FirmwareInfoType::Clear as u8);
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
