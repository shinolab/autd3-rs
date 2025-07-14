use super::OperationGenerator;
use crate::{
    datagram::PhaseCorrection,
    error::AUTDDriverError,
    firmware::driver::{Operation, Version},
};

use autd3_core::{
    gain::Phase,
    geometry::{Device, Transducer},
};

enum Inner<F> {
    V10(crate::firmware::v10::operation::PhaseCorrectionOp<F>),
    V11(crate::firmware::v11::operation::PhaseCorrectionOp<F>),
    V12(crate::firmware::v12::operation::PhaseCorrectionOp<F>),
    V12_1(crate::firmware::v12_1::operation::PhaseCorrectionOp<F>),
}

pub struct PhaseCorrectionOp<F> {
    inner: Inner<F>,
}

impl<'a, F: Fn(&'a Transducer) -> Phase + Send + Sync> Operation<'a> for PhaseCorrectionOp<F> {
    type Error = AUTDDriverError;

    fn pack(&mut self, device: &'a Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut self.inner {
            Inner::V10(inner) => Operation::pack(inner, device, tx)?,
            Inner::V11(inner) => Operation::pack(inner, device, tx)?,
            Inner::V12(inner) => Operation::pack(inner, device, tx)?,
            Inner::V12_1(inner) => Operation::pack(inner, device, tx)?,
        })
    }

    fn required_size(&self, device: &'a Device) -> usize {
        match &self.inner {
            Inner::V10(inner) => Operation::required_size(inner, device),
            Inner::V11(inner) => Operation::required_size(inner, device),
            Inner::V12(inner) => Operation::required_size(inner, device),
            Inner::V12_1(inner) => Operation::required_size(inner, device),
        }
    }

    fn is_done(&self) -> bool {
        match &self.inner {
            Inner::V10(inner) => Operation::is_done(inner),
            Inner::V11(inner) => Operation::is_done(inner),
            Inner::V12(inner) => Operation::is_done(inner),
            Inner::V12_1(inner) => Operation::is_done(inner),
        }
    }
}

impl<'a, FT: Fn(&'a Transducer) -> Phase + Send + Sync, F: Fn(&'a Device) -> FT>
    OperationGenerator<'a> for PhaseCorrection<F, FT>
{
    type O1 = PhaseCorrectionOp<FT>;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &'a Device, version: Version) -> Option<(Self::O1, Self::O2)> {
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
                    Version::V12_1 => Inner::V12_1(
                        crate::firmware::v12_1::operation::OperationGenerator::generate(
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
