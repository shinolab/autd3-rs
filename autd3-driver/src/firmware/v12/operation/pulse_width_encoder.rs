use super::{super::fpga::ULTRASOUND_PERIOD_COUNT_BITS, Operation, OperationGenerator};
use crate::{
    datagram::PulseWidthEncoder,
    firmware::v11::operation::{
        Operation as OperationV11, OperationGenerator as OperationGeneratorV11, PulseWidthEncoderOp,
    },
    geometry::Device,
};

use autd3_core::{datagram::PulseWidth, gain::Intensity};

impl<F: Fn(Intensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u16> + Send + Sync> Operation
    for PulseWidthEncoderOp<F>
{
    type Error = <Self as OperationV11>::Error;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        OperationV11::pack(self, device, tx)
    }

    fn required_size(&self, device: &Device) -> usize {
        OperationV11::required_size(self, device)
    }

    fn is_done(&self) -> bool {
        OperationV11::is_done(self)
    }
}

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
