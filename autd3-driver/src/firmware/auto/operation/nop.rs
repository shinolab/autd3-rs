use super::OperationGenerator;
use crate::{
    datagram::Nop,
    error::AUTDDriverError,
    firmware::driver::{Operation, Version},
    geometry::Device,
};

enum Inner {
    V10,
    V11,
    V12(crate::firmware::v12::operation::NopOp),
}

pub struct NopOp {
    inner: Inner,
}

impl Operation for NopOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut self.inner {
            Inner::V10 | Inner::V11 => return Err(AUTDDriverError::UnsupportedOperation),
            Inner::V12(inner) => Operation::pack(inner, device, tx)?,
        })
    }

    fn required_size(&self, device: &Device) -> usize {
        match &self.inner {
            Inner::V12(inner) => Operation::required_size(inner, device),
            _ => 0,
        }
    }

    fn is_done(&self) -> bool {
        match &self.inner {
            Inner::V12(inner) => Operation::is_done(inner),
            _ => false,
        }
    }
}

impl OperationGenerator for Nop {
    type O1 = NopOp;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        Some((
            NopOp {
                inner: match version {
                    Version::V10 => Inner::V10,
                    Version::V11 => Inner::V11,
                    Version::V12 => Inner::V12(
                        crate::firmware::v12::operation::OperationGenerator::generate(
                            self, device,
                        )?
                        .0,
                    ),
                },
            },
            crate::firmware::driver::NullOp,
        ))
    }
}
