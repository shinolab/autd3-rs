use std::time::Duration;

use crate::{
    datagram::*,
    derive::{AUTDInternalError, Geometry},
    firmware::operation::{FirmInfoOp, FirmwareVersionType},
};

use super::OperationGenerator;

pub struct FetchFirmwareInfoOpGenerator {
    inner: FirmwareVersionType,
}

impl OperationGenerator for FetchFirmwareInfoOpGenerator {
    type O1 = FirmInfoOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.inner), Self::O2::default())
    }
}

impl Datagram for FirmwareVersionType {
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
