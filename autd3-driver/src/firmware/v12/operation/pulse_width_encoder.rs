use super::{super::fpga::ULTRASOUND_PERIOD_COUNT_BITS, OperationGenerator};
use crate::{
    datagram::PulseWidthEncoder,
    firmware::v11::operation::OperationGenerator as OperationGeneratorV11, geometry::Device,
};

use autd3_core::{datagram::PulseWidth, gain::Intensity};

impl<
    H: Fn(Intensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u16> + Send + Sync,
    F: Fn(&Device) -> H,
> OperationGenerator for PulseWidthEncoder<ULTRASOUND_PERIOD_COUNT_BITS, u16, H, F>
{
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = <Self as OperationGeneratorV11>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
