use std::convert::Infallible;

use crate::{
    datagram::*,
    firmware::operation::{FirmInfoOp, FirmwareVersionType},
    geometry::Geometry,
};

use super::OperationGenerator;

pub struct FetchFirmwareInfoOpGenerator {
    inner: FirmwareVersionType,
}

impl OperationGenerator for FetchFirmwareInfoOpGenerator {
    type O1 = FirmInfoOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(self.inner), Self::O2 {}))
    }
}

impl Datagram for FirmwareVersionType {
    type G = FetchFirmwareInfoOpGenerator;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: &DeviceFilter) -> Result<Self::G, Self::Error> {
        Ok(Self::G { inner: self })
    }
}
