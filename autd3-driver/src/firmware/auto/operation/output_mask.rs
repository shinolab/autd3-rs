use super::OperationGenerator;
use crate::{
    datagram::OutputMaskOperationGenerator,
    error::AUTDDriverError,
    firmware::driver::{Operation, Version},
};

use autd3_core::geometry::{Device, Transducer};

enum Inner<F> {
    V10,
    V11,
    V12,
    V12_1(crate::firmware::v12_1::operation::OutputMaskOp<F>),
}

pub struct OutputMaskOp<F> {
    inner: Inner<F>,
}

impl<'a, FT: Fn(&'a Transducer) -> bool + Send + Sync> Operation<'a> for OutputMaskOp<FT> {
    type Error = AUTDDriverError;

    fn pack(&mut self, device: &'a Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut self.inner {
            Inner::V10 | Inner::V11 | Inner::V12 => {
                return Err(AUTDDriverError::UnsupportedOperation);
            }
            Inner::V12_1(inner) => Operation::pack(inner, device, tx)?,
        })
    }

    fn required_size(&self, device: &'a Device) -> usize {
        match &self.inner {
            Inner::V12_1(inner) => Operation::required_size(inner, device),
            _ => 0,
        }
    }

    fn is_done(&self) -> bool {
        match &self.inner {
            Inner::V12_1(inner) => Operation::is_done(inner),
            _ => false,
        }
    }
}

impl<'a, FT: Fn(&'a Transducer) -> bool + Send + Sync, F: Fn(&'a Device) -> FT>
    OperationGenerator<'a> for OutputMaskOperationGenerator<F>
{
    type O1 = OutputMaskOp<FT>;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &'a Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        Some((
            OutputMaskOp {
                inner: match version {
                    Version::V10 => Inner::V10,
                    Version::V11 => Inner::V11,
                    Version::V12 => Inner::V12,
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
