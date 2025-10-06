use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    geometry::Geometry,
};

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

pub struct FetchFirmwareInfoOpGenerator {
    pub(crate) inner: FirmwareVersionType,
}

impl Datagram<'_> for FirmwareVersionType {
    type G = FetchFirmwareInfoOpGenerator;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceMask,
    ) -> Result<Self::G, Self::Error> {
        Ok(Self::G { inner: self })
    }
}
