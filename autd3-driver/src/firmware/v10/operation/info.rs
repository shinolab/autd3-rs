use std::convert::Infallible;

use super::OperationGenerator;
use crate::{
    datagram::{FetchFirmwareInfoOpGenerator, FirmwareVersionType},
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
};

use autd3_core::geometry::Device;

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct FirmInfo {
    tag: TypeTag,
    ty: FirmwareVersionType,
}

pub struct FirmInfoOp {
    is_done: bool,
    ty: FirmwareVersionType,
}

impl FirmInfoOp {
    pub(crate) const fn new(ty: FirmwareVersionType) -> Self {
        Self { is_done: false, ty }
    }
}

impl Operation<'_> for FirmInfoOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
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

impl OperationGenerator<'_> for FetchFirmwareInfoOpGenerator {
    type O1 = FirmInfoOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(self.inner), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case(FirmwareVersionType::CPUMajor)]
    #[case(FirmwareVersionType::CPUMinor)]
    #[case(FirmwareVersionType::FPGAMajor)]
    #[case(FirmwareVersionType::FPGAMinor)]
    #[case(FirmwareVersionType::FPGAFunctions)]
    #[case(FirmwareVersionType::Clear)]
    fn test(#[case] ty: FirmwareVersionType) {
        let device = crate::autd3_device::tests::create_device();

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
