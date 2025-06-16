use autd3_core::{gain::Phase, geometry::Transducer};

use super::{super::Version, Operation, OperationGenerator};
use crate::{datagram::PhaseCorrection, error::AUTDDriverError, geometry::Device};

enum Inner<F: Fn(&Transducer) -> Phase> {
    V10(crate::firmware::v10::operation::PhaseCorrectionOp<F>),
    V11(crate::firmware::v11::operation::PhaseCorrectionOp<F>),
    V12(crate::firmware::v12::operation::PhaseCorrectionOp<F>),
}

pub struct PhaseCorrectionOp<F: Fn(&Transducer) -> Phase> {
    inner: Inner<F>,
}

impl<F: Fn(&Transducer) -> Phase + Send + Sync> Operation for PhaseCorrectionOp<F> {
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

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> OperationGenerator
    for PhaseCorrection<FT, F>
{
    type O1 = PhaseCorrectionOp<FT>;
    type O2 = super::NullOp;

    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        Some((
            PhaseCorrectionOp {
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
