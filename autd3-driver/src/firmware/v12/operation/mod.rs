mod fpga_gpio_out;
mod nop;

pub(crate) use crate::firmware::v11::operation::*;
pub(crate) use nop::NopOp;

use crate::{firmware::driver::Operation, geometry::Device};

#[doc(hidden)]
pub trait OperationGenerator {
    type O1: Operation;
    type O2: Operation;

    #[must_use]
    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)>;
}

macro_rules! impl_v12_op {
    ($ty:ty) => {
        impl OperationGenerator for $ty {
            type O1 = <Self as crate::firmware::v11::operation::OperationGenerator>::O1;
            type O2 = <Self as crate::firmware::v11::operation::OperationGenerator>::O2;

            fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
                crate::firmware::v11::operation::OperationGenerator::generate(self, device)
            }
        }
    };
    ($($generics:tt),+; $ty:ty) => {
           impl<$($generics),+> OperationGenerator for $ty
           where
               $ty: crate::firmware::v11::operation::OperationGenerator,
           {
               type O1 = <Self as crate::firmware::v11::operation::OperationGenerator>::O1;
               type O2 = <Self as crate::firmware::v11::operation::OperationGenerator>::O2;

               fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
                   crate::firmware::v11::operation::OperationGenerator::generate(self, device)
               }
           }
    };
}

impl_v12_op!(crate::datagram::FixedCompletionSteps);
impl_v12_op!(crate::datagram::FixedCompletionTime);
impl_v12_op!(crate::datagram::FixedUpdateRate);
impl_v12_op!(crate::datagram::Clear);
impl_v12_op!(crate::datagram::Synchronize);
impl_v12_op!(crate::datagram::SwapSegment);
impl_v12_op!(crate::datagram::FetchFirmwareInfoOpGenerator);
impl_v12_op!(autd3_core::modulation::ModulationOperationGenerator);
impl_v12_op!(G; autd3_core::gain::GainOperationGenerator<G>);
impl_v12_op!(F; crate::datagram::CpuGPIOOutputs<F>);
impl_v12_op!(F; crate::datagram::ForceFan<F>);
impl_v12_op!(F; crate::datagram::PulseWidthEncoder<autd3_core::datagram::PulseWidth<{ super::fpga::ULTRASOUND_PERIOD_COUNT_BITS }, u16>, F>);
impl_v12_op!(F; crate::datagram::ReadsFPGAState<F>);
impl_v12_op!(F; crate::datagram::PhaseCorrection<F>);
impl_v12_op!(F; crate::datagram::EmulateGPIOIn<F>);
impl_v12_op!(T; crate::datagram::GainSTMOperationGenerator<T>);
impl_v12_op!(O1, O2; autd3_core::datagram::CombinedOperationGenerator<O1, O2>);
impl_v12_op!(K, F, G; crate::datagram::GroupOpGenerator<K, F, G>);

impl<const N: usize, G: crate::datagram::FociSTMIteratorGenerator<N>> OperationGenerator
    for crate::datagram::FociSTMOperationGenerator<N, G>
{
    type O1 = <Self as crate::firmware::v11::operation::OperationGenerator>::O1;
    type O2 = <Self as crate::firmware::v11::operation::OperationGenerator>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v11::operation::OperationGenerator::generate(self, device)
    }
}

impl OperationGenerator for crate::firmware::driver::DynOperationGenerator {
    type O1 = crate::firmware::driver::BoxedOperation;
    type O2 = crate::firmware::driver::BoxedOperation;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        self.g
            .dyn_generate(device, crate::firmware::driver::Version::V12)
    }
}
