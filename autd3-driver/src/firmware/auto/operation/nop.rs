use super::{super::Version, Operation, OperationGenerator};
use crate::{datagram::Nop, error::AUTDDriverError, geometry::Device};

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
            Inner::V12(inner) => {
                crate::firmware::v12::operation::Operation::pack(inner, device, tx)?
            }
        })
    }

    fn required_size(&self, device: &Device) -> usize {
        match &self.inner {
            Inner::V12(inner) => {
                crate::firmware::v12::operation::Operation::required_size(inner, device)
            }
            _ => 0,
        }
    }

    fn is_done(&self) -> bool {
        match &self.inner {
            Inner::V12(inner) => crate::firmware::v12::operation::Operation::is_done(inner),
            _ => false,
        }
    }
}

impl OperationGenerator for Nop {
    type O1 = NopOp;
    type O2 = super::NullOp;

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
            super::NullOp,
        ))
    }
}
