use crate::{
    error::AUTDInternalError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use super::write_to_tx;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum FirmwareVersionType2 {
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
    ty: FirmwareVersionType2,
}

pub struct FirmInfoOp2 {
    is_done: bool,
    ty: FirmwareVersionType2,
}

impl FirmInfoOp2 {
    pub(crate) fn new(ty: FirmwareVersionType2) -> Self {
        Self { is_done: false, ty }
    }
}

impl Operation for FirmInfoOp2 {
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
    #[case(FirmwareVersionType2::CPUVersionMajor)]
    #[case(FirmwareVersionType2::CPUVersionMinor)]
    #[case(FirmwareVersionType2::FPGAVersionMajor)]
    #[case(FirmwareVersionType2::FPGAVersionMinor)]
    #[case(FirmwareVersionType2::FPGAFunctions)]
    #[case(FirmwareVersionType2::Clear)]
    fn test(#[case] ty: FirmwareVersionType2) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<FirmInfo>()];

        let mut op = FirmInfoOp2::new(ty);

        assert_eq!(op.required_size(&device), size_of::<FirmInfo>());
        assert!(!op.is_done);

        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done);
        assert_eq!(tx[0], TypeTag::FirmwareVersion as u8);
        assert_eq!(tx[1], ty as u8);
    }
}
