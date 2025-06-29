use super::OperationGenerator;
use crate::{
    datagram::OutputMask,
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

impl<FT: Fn(&Transducer) -> bool + Send + Sync> Operation for OutputMaskOp<FT> {
    type Error = AUTDDriverError;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut self.inner {
            Inner::V10 | Inner::V11 | Inner::V12 => {
                return Err(AUTDDriverError::UnsupportedOperation);
            }
            Inner::V12_1(inner) => Operation::pack(inner, device, tx)?,
        })
    }

    fn required_size(&self, device: &Device) -> usize {
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

impl<FT: Fn(&Transducer) -> bool + Send + Sync, F: Fn(&Device) -> FT> OperationGenerator
    for OutputMask<F>
{
    type O1 = OutputMaskOp<FT>;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)> {
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
