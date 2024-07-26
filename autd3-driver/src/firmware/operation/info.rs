use crate::{
    error::AUTDInternalError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use super::write_to_tx;

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

pub struct FirmInfoOp {
    remains: usize,
}

impl Default for FirmInfoOp {
    fn default() -> Self {
        Self { remains: 6 }
    }
}

impl Operation for FirmInfoOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        write_to_tx(
            FirmInfo {
                tag: TypeTag::FirmwareVersion,
                ty: match self.remains {
                    6 => FirmwareVersionType::CPUVersionMajor,
                    5 => FirmwareVersionType::CPUVersionMinor,
                    4 => FirmwareVersionType::FPGAVersionMajor,
                    3 => FirmwareVersionType::FPGAVersionMinor,
                    2 => FirmwareVersionType::FPGAFunctions,
                    1 => FirmwareVersionType::Clear,
                    _ => unreachable!(),
                },
            },
            tx,
        );

        self.remains -= 1;
        Ok(std::mem::size_of::<FirmInfo>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<FirmInfo>()
    }

    fn is_done(&self) -> bool {
        self.remains == 0
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

        let mut tx = [0x00u8; size_of::<FirmInfo>()];

        let mut op = FirmInfoOp::default();

        assert_eq!(op.required_size(&device), size_of::<FirmInfo>());
        assert_eq!(op.remains, 6);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert_eq!(op.remains, 5);
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], FirmwareVersionType::CPUVersionMajor as u8);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert_eq!(op.remains, 4);
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], FirmwareVersionType::CPUVersionMinor as u8);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert_eq!(op.remains, 3);
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], FirmwareVersionType::FPGAVersionMajor as u8);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert_eq!(op.remains, 2);
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], FirmwareVersionType::FPGAVersionMinor as u8);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert_eq!(op.remains, 1);
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], FirmwareVersionType::FPGAFunctions as u8);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done());
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], FirmwareVersionType::Clear as u8);
    }

    #[test]
    #[should_panic]
    #[cfg_attr(miri, ignore)]
    fn test_panic() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = [0x00u8; size_of::<FirmInfo>()];

        let mut op = FirmInfoOp::default();

        (0..7).for_each(|_| {
            assert!(op.pack(&device, &mut tx[0..]).is_ok());
        });
    }
}
