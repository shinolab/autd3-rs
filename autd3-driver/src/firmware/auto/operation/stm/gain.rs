use autd3_core::{
    gain::{GainCalculator, GainCalculatorGenerator},
    geometry::Device,
};

use super::super::OperationGenerator;
use crate::{
    datagram::{GainSTMIterator, GainSTMIteratorGenerator, GainSTMOperationGenerator},
    error::AUTDDriverError,
    firmware::driver::{Operation, Version},
};

enum Inner<G, Iterator> {
    V10(crate::firmware::v10::operation::GainSTMOp<G, Iterator>),
    V11(crate::firmware::v11::operation::GainSTMOp<G, Iterator>),
    V12(crate::firmware::v12::operation::GainSTMOp<G, Iterator>),
    V12_1(crate::firmware::v12_1::operation::GainSTMOp<G, Iterator>),
}

pub struct GainSTMOp<G, Iterator> {
    inner: Inner<G, Iterator>,
}

impl<'a, G: GainCalculator<'a>, Iterator: GainSTMIterator<'a, Calculator = G>> Operation<'a>
    for GainSTMOp<G, Iterator>
{
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

impl<'a, G: GainSTMIteratorGenerator<'a>> OperationGenerator<'a>
    for GainSTMOperationGenerator<'a, G>
{
    type O1 = GainSTMOp<<G::Gain as GainCalculatorGenerator<'a>>::Calculator, G::Iterator>;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &'a Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        Some((
            GainSTMOp {
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
