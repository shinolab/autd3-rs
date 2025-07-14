mod output_mask;

pub(crate) use crate::firmware::v12::operation::*;
pub(crate) use output_mask::OutputMaskOp;

use crate::firmware::driver::Operation;

use autd3_core::geometry::Device;

#[doc(hidden)]
pub trait OperationGenerator<'a> {
    type O1: Operation<'a>;
    type O2: Operation<'a>;

    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)>;
}

macro_rules! impl_v12_1_op {
    ($ty:ty) => {
        impl<'a> OperationGenerator<'a> for  $ty {
            type O1 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O1;
            type O2 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O2;

            fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
                crate::firmware::v12::operation::OperationGenerator::generate(self, device)
            }
        }
    };
    ($($generics:tt),+; $ty:ty) => {
           impl<'a, $($generics),+> OperationGenerator<'a> for  $ty
           where
               $ty: crate::firmware::v12::operation::OperationGenerator<'a>,
           {
               type O1 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O1;
               type O2 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O2;

               fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
                   crate::firmware::v12::operation::OperationGenerator::generate(self, device)
               }
           }
    };
}

impl_v12_1_op!(crate::datagram::FixedCompletionSteps);
impl_v12_1_op!(crate::datagram::FixedCompletionTime);
impl_v12_1_op!(crate::datagram::FixedUpdateRate);
impl_v12_1_op!(crate::datagram::Clear);
impl_v12_1_op!(crate::datagram::Nop);
impl_v12_1_op!(crate::datagram::Synchronize);
impl_v12_1_op!(crate::datagram::SwapSegment);
impl_v12_1_op!(crate::datagram::FetchFirmwareInfoOpGenerator);
impl_v12_1_op!(autd3_core::modulation::ModulationOperationGenerator);
impl_v12_1_op!(F; crate::datagram::CpuGPIOOutputs<F>);
impl_v12_1_op!(F; crate::datagram::GPIOOutputs<F>);
impl_v12_1_op!(F; crate::datagram::ForceFan<F>);
impl_v12_1_op!(F; crate::datagram::PulseWidthEncoderOperationGenerator<F>);
impl_v12_1_op!(F; crate::datagram::ReadsFPGAState<F>);
impl_v12_1_op!(F; crate::datagram::EmulateGPIOIn<F>);
impl_v12_1_op!(F, FT; crate::datagram::PhaseCorrection<F, FT>);
impl_v12_1_op!(O1, O2; autd3_core::datagram::CombinedOperationGenerator<O1, O2>);
impl_v12_1_op!(K, F, G; crate::datagram::GroupOpGenerator<K, F, G>);

impl<'a, G> OperationGenerator<'a> for autd3_core::gain::GainOperationGenerator<'a, G>
where
    autd3_core::gain::GainOperationGenerator<'a, G>:
        crate::firmware::v12::operation::OperationGenerator<'a>,
{
    type O1 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O1;
    type O2 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O2;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v12::operation::OperationGenerator::generate(self, device)
    }
}

impl<'a, T> OperationGenerator<'a> for crate::datagram::GainSTMOperationGenerator<'a, T>
where
    crate::datagram::GainSTMOperationGenerator<'a, T>:
        crate::firmware::v12::operation::OperationGenerator<'a>,
{
    type O1 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O1;
    type O2 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O2;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v12::operation::OperationGenerator::generate(self, device)
    }
}

impl<'a, const N: usize, G: crate::datagram::FociSTMIteratorGenerator<N>> OperationGenerator<'a>
    for crate::datagram::FociSTMOperationGenerator<N, G>
{
    type O1 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O1;
    type O2 = <Self as crate::firmware::v12::operation::OperationGenerator<'a>>::O2;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v12::operation::OperationGenerator::generate(self, device)
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
