use autd3_core::{datagram::PulseWidth, gain::Intensity};

use super::{super::Version, Operation, OperationGenerator};
use crate::{datagram::PulseWidthEncoder, geometry::Device};

use crate::firmware::v10::operation::{
    Operation as OperationV10, OperationGenerator as OperationGeneratorV10,
};
use crate::firmware::v11::operation::{
    Operation as OperationV11, OperationGenerator as OperationGeneratorV11,
};

impl<H: Fn(Intensity) -> PulseWidth<9, u16> + Send + Sync> Operation
    for crate::firmware::v11::operation::PulseWidthEncoderOp<H>
{
    type Error = <Self as OperationV11>::Error;

    fn required_size(&self, device: &Device) -> usize {
        <Self as OperationV11>::required_size(self, device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        <Self as OperationV11>::pack(self, device, tx)
    }

    fn is_done(&self) -> bool {
        <Self as OperationV11>::is_done(self)
    }
}

impl<H: Fn(Intensity) -> PulseWidth<8, u8> + Send + Sync> Operation
    for crate::firmware::v10::operation::PulseWidthEncoderOp<H>
{
    type Error = <Self as OperationV10>::Error;

    fn required_size(&self, device: &Device) -> usize {
        <Self as OperationV10>::required_size(self, device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        <Self as OperationV10>::pack(self, device, tx)
    }

    fn is_done(&self) -> bool {
        <Self as OperationV10>::is_done(self)
    }
}

impl<H: Fn(Intensity) -> PulseWidth<9, u16> + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for PulseWidthEncoder<9, u16, H, F>
{
    type O1 = crate::firmware::v11::operation::PulseWidthEncoderOp<H>;
    type O2 = super::NullOp;

    fn generate(&mut self, device: &Device, _: Version) -> Option<(Self::O1, Self::O2)> {
        <Self as OperationGeneratorV11>::generate(self, device).map(|(op, _)| (op, super::NullOp))
    }
}

impl<H: Fn(Intensity) -> PulseWidth<8, u8> + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for PulseWidthEncoder<8, u8, H, F>
{
    type O1 = crate::firmware::v10::operation::PulseWidthEncoderOp<H>;
    type O2 = super::NullOp;

    fn generate(&mut self, device: &Device, _: Version) -> Option<(Self::O1, Self::O2)> {
        <Self as OperationGeneratorV10>::generate(self, device).map(|(op, _)| (op, super::NullOp))
    }
}
