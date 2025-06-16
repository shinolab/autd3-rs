use super::{super::Version, Operation, OperationGenerator};
use crate::{datagram::Synchronize, error::AUTDDriverError, geometry::Device};

enum Inner {
    V10(crate::firmware::v10::operation::SyncOp),
    V11(crate::firmware::v11::operation::SyncOp),
    V12(crate::firmware::v12::operation::SyncOp),
}

pub struct SyncOp {
    inner: Inner,
}

impl Operation for SyncOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut self.inner {
            Inner::V10(inner) => {
                crate::firmware::v10::operation::Operation::pack(inner, device, tx)?
            }
            Inner::V11(inner) => {
                crate::firmware::v11::operation::Operation::pack(inner, device, tx)?
            }
            Inner::V12(inner) => {
                crate::firmware::v12::operation::Operation::pack(inner, device, tx)?
            }
        })
    }

    fn required_size(&self, device: &Device) -> usize {
        match &self.inner {
            Inner::V10(inner) => {
                crate::firmware::v10::operation::Operation::required_size(inner, device)
            }
            Inner::V11(inner) => {
                crate::firmware::v11::operation::Operation::required_size(inner, device)
            }
            Inner::V12(inner) => {
                crate::firmware::v12::operation::Operation::required_size(inner, device)
            }
        }
    }

    fn is_done(&self) -> bool {
        match &self.inner {
            Inner::V10(inner) => crate::firmware::v10::operation::Operation::is_done(inner),
            Inner::V11(inner) => crate::firmware::v11::operation::Operation::is_done(inner),
            Inner::V12(inner) => crate::firmware::v12::operation::Operation::is_done(inner),
        }
    }
}

impl OperationGenerator for Synchronize {
    type O1 = SyncOp;
    type O2 = super::NullOp;

    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        Some((
            SyncOp {
                inner: match version {
                    Version::V10 => Inner::V10(
                        crate::firmware::v10::operation::OperationGenerator::generate(
                            self, device,
                        )?
                        .0,
                    ),
                    Version::V11 => Inner::V11(
                        crate::firmware::v11::operation::OperationGenerator::generate(
                            self, device,
                        )?
                        .0,
                    ),
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
