use std::time::Duration;

use crate::{
    datagram::*,
    derive::{AUTDInternalError, Geometry},
    firmware::operation::{FirmInfoOp2, FirmwareVersionType2},
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
    type O1 = FirmInfoOp2;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(match self.inner {
                FetchFirmInfo::CPUMajor => FirmwareVersionType2::CPUVersionMajor,
                FetchFirmInfo::CPUMinor => FirmwareVersionType2::CPUVersionMinor,
                FetchFirmInfo::FPGAMajor => FirmwareVersionType2::FPGAVersionMajor,
                FetchFirmInfo::FPGAMinor => FirmwareVersionType2::FPGAVersionMinor,
                FetchFirmInfo::FPGAFunctions => FirmwareVersionType2::FPGAFunctions,
                FetchFirmInfo::Clear => FirmwareVersionType2::Clear,
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

    #[tracing::instrument(skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}
