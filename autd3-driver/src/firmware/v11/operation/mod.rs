mod pulse_width_encoder;

pub(crate) use crate::firmware::v10::operation::*;
pub(crate) use pulse_width_encoder::PulseWidthEncoderOp;

use crate::firmware::driver::Operation;

use autd3_core::geometry::Device;

#[doc(hidden)]
pub trait OperationGenerator<'dev> {
    type O1: Operation<'dev>;
    type O2: Operation<'dev>;

    #[must_use]
    fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)>;
}

macro_rules! impl_v11_op {
    ($ty:ty) => {
        impl<'dev> OperationGenerator<'dev> for  $ty {
            type O1 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O1;
            type O2 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O2;

            fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)> {
                crate::firmware::v10::operation::OperationGenerator::generate(self, device)
            }
        }
    };
    ($($generics:tt),*; $ty:ty) => {
           impl<'dev, $($generics),*> OperationGenerator<'dev> for $ty
           where
               $ty: crate::firmware::v10::operation::OperationGenerator<'dev>,
           {
               type O1 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O1;
               type O2 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O2;

               fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)> {
                   crate::firmware::v10::operation::OperationGenerator::generate(self, device)
               }
           }
    };
}

impl_v11_op!(crate::datagram::FixedCompletionSteps);
impl_v11_op!(crate::datagram::FixedCompletionTime);
impl_v11_op!(crate::datagram::FixedUpdateRate);
impl_v11_op!(crate::datagram::Clear);
impl_v11_op!(crate::datagram::Synchronize);
impl_v11_op!(crate::datagram::SwapSegment);
impl_v11_op!(crate::datagram::FetchFirmwareInfoOpGenerator);
impl_v11_op!(autd3_core::modulation::ModulationOperationGenerator);
impl_v11_op!(F; crate::datagram::CpuGPIOOutputs<F>);
impl_v11_op!(F; crate::datagram::ForceFan<F>);
impl_v11_op!(F; crate::datagram::ReadsFPGAState<F>);
impl_v11_op!(F; crate::datagram::GPIOOutputs<F>);
impl_v11_op!(F; crate::datagram::EmulateGPIOIn<F>);
impl_v11_op!(F, FT; crate::datagram::PhaseCorrection<F, FT>);
impl_v11_op!(O1, O2; autd3_core::datagram::CombinedOperationGenerator<O1, O2>);
impl_v11_op!(K, F, G; crate::datagram::GroupOpGenerator<K, F, G>);

impl<'dev, 'tr, G> OperationGenerator<'dev> for autd3_core::gain::GainOperationGenerator<'tr, G>
where
    autd3_core::gain::GainOperationGenerator<'tr, G>:
        crate::firmware::v10::operation::OperationGenerator<'dev>,
{
    type O1 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O1;
    type O2 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O2;

    fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v10::operation::OperationGenerator::generate(self, device)
    }
}

impl<'dev, 'tr, T> OperationGenerator<'dev> for crate::datagram::GainSTMOperationGenerator<'tr, T>
where
    crate::datagram::GainSTMOperationGenerator<'tr, T>:
        crate::firmware::v10::operation::OperationGenerator<'dev>,
{
    type O1 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O1;
    type O2 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O2;

    fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v10::operation::OperationGenerator::generate(self, device)
    }
}

impl<'dev, const N: usize, G: crate::datagram::FociSTMIteratorGenerator<N>> OperationGenerator<'dev>
    for crate::datagram::FociSTMOperationGenerator<N, G>
{
    type O1 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O1;
    type O2 = <Self as crate::firmware::v10::operation::OperationGenerator<'dev>>::O2;

    fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)> {
        crate::firmware::v10::operation::OperationGenerator::generate(self, device)
    }
}

impl<'dev> OperationGenerator<'dev> for crate::firmware::driver::DynOperationGenerator {
    type O1 = crate::firmware::driver::BoxedOperation;
    type O2 = crate::firmware::driver::BoxedOperation;

    fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)> {
        self.g
            .dyn_generate(device, crate::firmware::driver::Version::V11)
    }
}
