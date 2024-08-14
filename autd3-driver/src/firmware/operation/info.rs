// GRCOV_EXCL_START

#![allow(deprecated)]

use crate::{
    error::AUTDInternalError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use super::write_to_tx;

#[deprecated(note = "Use FirmwareVersionType2 instead")]
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

#[deprecated(note = "Use FirmInfoOp2 instead")]
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

// GRPCOV_EXCL_STOP
