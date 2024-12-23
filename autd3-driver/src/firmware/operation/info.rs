use crate::{
    error::AUTDDriverError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntoBytes, Immutable)]
#[doc(hidden)]
pub enum FirmwareVersionType {
    CPUMajor = 0x01,
    CPUMinor = 0x02,
    FPGAMajor = 0x03,
    FPGAMinor = 0x04,
    FPGAFunctions = 0x05,
    Clear = 0x06,
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct FirmInfo {
    tag: TypeTag,
    ty: FirmwareVersionType,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct FirmInfoOp {
    #[new(default)]
    is_done: bool,
    ty: FirmwareVersionType,
}

impl Operation for FirmInfoOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        super::write_to_tx(
            tx,
            FirmInfo {
                tag: TypeTag::FirmwareVersion,
                ty: self.ty,
            },
        );
        self.is_done = true;
        Ok(size_of::<FirmInfo>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<FirmInfo>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[rstest::rstest]
    #[test]
    #[case(FirmwareVersionType::CPUMajor)]
    #[case(FirmwareVersionType::CPUMinor)]
    #[case(FirmwareVersionType::FPGAMajor)]
    #[case(FirmwareVersionType::FPGAMinor)]
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
