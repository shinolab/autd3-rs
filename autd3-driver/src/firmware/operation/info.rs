use crate::{
    error::AUTDInternalError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use super::write_to_tx;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum FirmwareVersionType {
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
    is_done: bool,
    ty: FirmwareVersionType,
}

impl FirmInfoOp {
    pub(crate) fn new(ty: FirmwareVersionType) -> Self {
        Self { is_done: false, ty }
    }
}

impl Operation for FirmInfoOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        write_to_tx(
            FirmInfo {
                tag: TypeTag::FirmwareVersion,
                ty: self.ty,
            },
            tx,
        );

        self.is_done = true;
        Ok(std::mem::size_of::<FirmInfo>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<FirmInfo>()
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

    #[rstest::rstest]
    #[test]
    #[case(FirmwareVersionType::CPUVersionMajor)]
    #[case(FirmwareVersionType::CPUVersionMinor)]
    #[case(FirmwareVersionType::FPGAVersionMajor)]
    #[case(FirmwareVersionType::FPGAVersionMinor)]
    #[case(FirmwareVersionType::FPGAFunctions)]
    #[case(FirmwareVersionType::Clear)]
    fn test(#[case] ty: FirmwareVersionType) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<FirmInfo>()];

        let mut op = FirmInfoOp::new(ty);

        assert_eq!(op.required_size(&device), size_of::<FirmInfo>());
        assert!(!op.is_done);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done);
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], ty as u8);
    }
}
