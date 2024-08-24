use std::time::Duration;

use crate::{
    datagram::*,
    derive::{AUTDInternalError, Geometry},
    firmware::operation::{FirmInfoOp, FirmwareVersionType},
};

use super::OperationGenerator;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum FetchFirmInfo {
    CPUMajor,
    CPUMinor,
    FPGAMajor,
    FPGAMinor,
    FPGAFunctions,
    Clear,
}

pub struct FetchFirmwareInfoOpGenerator {
    inner: FetchFirmInfo,
}

impl OperationGenerator for FetchFirmwareInfoOpGenerator {
    type O1 = FirmInfoOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(match self.inner {
                FetchFirmInfo::CPUMajor => FirmwareVersionType::CPUVersionMajor,
                FetchFirmInfo::CPUMinor => FirmwareVersionType::CPUVersionMinor,
                FetchFirmInfo::FPGAMajor => FirmwareVersionType::FPGAVersionMajor,
                FetchFirmInfo::FPGAMinor => FirmwareVersionType::FPGAVersionMinor,
                FetchFirmInfo::FPGAFunctions => FirmwareVersionType::FPGAFunctions,
                FetchFirmInfo::Clear => FirmwareVersionType::Clear,
            }),
            Self::O2::default(),
        )
    }
}

impl Datagram for FetchFirmInfo {
    type G = FetchFirmwareInfoOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G { inner: self })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
