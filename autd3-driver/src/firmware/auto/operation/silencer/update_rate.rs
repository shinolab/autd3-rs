use super::super::OperationGenerator;
use crate::{
    datagram::FixedUpdateRate,
    error::AUTDDriverError,
    firmware::driver::{Operation, Version},
    geometry::Device,
};

enum Inner {
    V10(crate::firmware::v10::operation::SilencerFixedUpdateRateOp),
    V11(crate::firmware::v11::operation::SilencerFixedUpdateRateOp),
    V12(crate::firmware::v12::operation::SilencerFixedUpdateRateOp),
}

pub struct SilencerFixedUpdateRateOp {
    inner: Inner,
}

impl Operation for SilencerFixedUpdateRateOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut self.inner {
            Inner::V10(inner) => Operation::pack(inner, device, tx)?,
            Inner::V11(inner) => Operation::pack(inner, device, tx)?,
            Inner::V12(inner) => Operation::pack(inner, device, tx)?,
        })
    }

    fn required_size(&self, device: &Device) -> usize {
        match &self.inner {
            Inner::V10(inner) => Operation::required_size(inner, device),
            Inner::V11(inner) => Operation::required_size(inner, device),
            Inner::V12(inner) => Operation::required_size(inner, device),
        }
    }

    fn is_done(&self) -> bool {
        match &self.inner {
            Inner::V10(inner) => Operation::is_done(inner),
            Inner::V11(inner) => Operation::is_done(inner),
            Inner::V12(inner) => Operation::is_done(inner),
        }
    }
}

impl OperationGenerator for FixedUpdateRate {
    type O1 = SilencerFixedUpdateRateOp;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        Some((
            SilencerFixedUpdateRateOp {
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
            crate::firmware::driver::NullOp,
        ))
    }
}
