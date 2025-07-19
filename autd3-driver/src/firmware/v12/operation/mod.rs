mod fpga_gpio_out;
mod nop;

pub(crate) use crate::firmware::v11::operation::*;
pub(crate) use nop::NopOp;

use crate::firmware::driver::Operation;

use autd3_core::geometry::Device;

#[doc(hidden)]
pub trait OperationGenerator<'a> {
    type O1: Operation<'a>;
    type O2: Operation<'a>;

    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)>;
}

macro_rules! impl_v12_op {
    ($ty:ty) => {
        impl<'a> OperationGenerator<'a> for  $ty {
            type O1 = <Self as crate::firmware::v11::operation::OperationGenerator<'a>>::O1;
            type O2 = <Self as crate::firmware::v11::operation::OperationGenerator<'a>>::O2;

            fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
                crate::firmware::v11::operation::OperationGenerator::generate(self, device)
            }
        }
    };
    ($($generics:tt),+; $ty:ty) => {
           impl<'a, $($generics),+> OperationGenerator<'a> for  $ty
           where
               $ty: crate::firmware::v11::operation::OperationGenerator<'a>,
           {
               type O1 = <Self as crate::firmware::v11::operation::OperationGenerator<'a>>::O1;
               type O2 = <Self as crate::firmware::v11::operation::OperationGenerator<'a>>::O2;

               fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
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
impl_v12_op!(crate::datagram::SwapSegmentGain);
impl_v12_op!(T; crate::datagram::SwapSegmentModulation<T>);
impl_v12_op!(T; crate::datagram::SwapSegmentFociSTM<T>);
impl_v12_op!(T; crate::datagram::SwapSegmentGainSTM<T>);
impl_v12_op!(crate::datagram::FetchFirmwareInfoOpGenerator);
impl_v12_op!(autd3_core::modulation::ModulationOperationGenerator);
impl_v12_op!(F; crate::datagram::CpuGPIOOutputs<F>);
impl_v12_op!(F; crate::datagram::ForceFan<F>);
impl_v12_op!(F; crate::datagram::PulseWidthEncoderOperationGenerator<F>);
impl_v12_op!(F; crate::datagram::ReadsFPGAState<F>);
impl_v12_op!(F; crate::datagram::EmulateGPIOIn<F>);
impl_v12_op!(F, FT; crate::datagram::PhaseCorrection<F, FT>);
impl_v12_op!(O1, O2; autd3_core::datagram::CombinedOperationGenerator<O1, O2>);
impl_v12_op!(G; autd3_core::gain::GainOperationGenerator<'a, G>);
impl_v12_op!(K, F, G; crate::datagram::GroupOpGenerator<K, F, G>);
impl_v12_op!(G; crate::datagram::GainSTMOperationGenerator<'a, G>);

impl<'a, const N: usize, G: crate::datagram::FociSTMIteratorGenerator<N>> OperationGenerator<'a>
    for crate::datagram::FociSTMOperationGenerator<N, G>
{
    type O1 = <Self as crate::firmware::v11::operation::OperationGenerator<'a>>::O1;
    type O2 = <Self as crate::firmware::v11::operation::OperationGenerator<'a>>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v11::operation::OperationGenerator::generate(self, device)
    }
}

impl<'a> OperationGenerator<'a> for crate::firmware::driver::DynOperationGenerator {
    type O1 = crate::firmware::driver::BoxedOperation;
    type O2 = crate::firmware::driver::BoxedOperation;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        self.g
            .dyn_generate(device, crate::firmware::driver::Version::V12)
    }
}
