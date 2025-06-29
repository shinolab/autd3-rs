use autd3_core::{datagram::PulseWidth, gain::Intensity, geometry::Device};

use super::OperationGenerator;
use crate::{datagram::PulseWidthEncoder, firmware::driver::Version};

use crate::firmware::v10::operation::OperationGenerator as OperationGeneratorV10;
use crate::firmware::v11::operation::OperationGenerator as OperationGeneratorV11;

impl<H: Fn(Intensity) -> PulseWidth<9, u16> + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for PulseWidthEncoder<PulseWidth<9, u16>, F>
{
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = <Self as OperationGeneratorV11>::O2;

    fn generate(&mut self, device: &Device, _: Version) -> Option<(Self::O1, Self::O2)> {
        <Self as OperationGeneratorV11>::generate(self, device)
    }
}

impl<H: Fn(Intensity) -> PulseWidth<8, u8> + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for PulseWidthEncoder<PulseWidth<8, u8>, F>
{
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device, _: Version) -> Option<(Self::O1, Self::O2)> {
        <Self as OperationGeneratorV10>::generate(self, device)
    }
}
