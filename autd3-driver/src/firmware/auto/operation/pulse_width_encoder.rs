use super::OperationGenerator;
use crate::{
    datagram::PulseWidthEncoderOperationGenerator,
    firmware::driver::{Operation, Version},
};

use autd3_core::{
    datagram::{PulseWidth, PulseWidthError},
    gain::Intensity,
    geometry::Device,
};

enum Inner<F: Fn(Intensity) -> PulseWidth> {
    V10(crate::firmware::v10::operation::PulseWidthEncoderOp<F>),
    V11(crate::firmware::v11::operation::PulseWidthEncoderOp<F>),
    V12(crate::firmware::v12::operation::PulseWidthEncoderOp<F>),
    V12_1(crate::firmware::v12_1::operation::PulseWidthEncoderOp<F>),
}

pub struct PulseWidthEncoderOp<F: Fn(Intensity) -> PulseWidth> {
    inner: Inner<F>,
}

impl<F: Fn(Intensity) -> PulseWidth + Send + Sync> Operation<'_> for PulseWidthEncoderOp<F> {
    type Error = PulseWidthError;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut self.inner {
            Inner::V10(inner) => Operation::pack(inner, device, tx)?,
            Inner::V11(inner) => Operation::pack(inner, device, tx)?,
            Inner::V12(inner) => Operation::pack(inner, device, tx)?,
            Inner::V12_1(inner) => Operation::pack(inner, device, tx)?,
        })
    }

    fn required_size(&self, device: &Device) -> usize {
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

impl<FT: Fn(Intensity) -> PulseWidth + Send + Sync, F: Fn(&Device) -> FT> OperationGenerator<'_>
    for PulseWidthEncoderOperationGenerator<F>
{
    type O1 = PulseWidthEncoderOp<FT>;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        Some((
            PulseWidthEncoderOp {
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
