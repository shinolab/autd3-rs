mod boxed;
mod gain;
mod group;
mod nop;
mod output_mask;
mod phase_corr;
mod pulse_width_encoder;
mod stm;
mod tuple;

use crate::firmware::driver::{Operation, Version};

use autd3_core::geometry::Device;

#[doc(hidden)]
pub trait OperationGenerator<'dev> {
    type O1: Operation<'dev>;
    type O2: Operation<'dev>;

    #[must_use]
    fn generate(&mut self, device: &'dev Device, version: Version) -> Option<(Self::O1, Self::O2)>;
}

macro_rules! impl_auto_op {
    ($op:ty,$gen:ty) => {
        paste::paste! {
            enum [<$op Inner>] {
                V10(crate::firmware::v10::operation::[<$op Op>]),
                V11(crate::firmware::v11::operation::[<$op Op>]),
                V12(crate::firmware::v12::operation::[<$op Op>]),
                V12_1(crate::firmware::v12_1::operation::[<$op Op>]),
            }

            #[doc(hidden)]
            pub struct [<$op Op>] {
                inner: [<$op Inner>] ,
            }

            impl Operation<'_,> for  [<$op Op>] {
                type Error = crate::error::AUTDDriverError;

                fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
                    Ok(match &mut self.inner {
                        [<$op Inner>]::V10(inner) => Operation::pack(inner, device, tx)?,
                        [<$op Inner>]::V11(inner) => Operation::pack(inner, device, tx)?,
                        [<$op Inner>]::V12(inner) => Operation::pack(inner, device, tx)?,
                        [<$op Inner>]::V12_1(inner) => Operation::pack(inner, device, tx)?,
                    })
                }

                fn required_size(&self, device: &Device) -> usize {
                    match &self.inner {
                        [<$op Inner>]::V10(inner) => Operation::required_size(inner, device),
                        [<$op Inner>]::V11(inner) => Operation::required_size(inner, device),
                        [<$op Inner>]::V12(inner) => Operation::required_size(inner, device),
                        [<$op Inner>]::V12_1(inner) => Operation::required_size(inner, device),
                    }
                }

                fn is_done(&self) -> bool {
                    match &self.inner {
                        [<$op Inner>]::V10(inner) => Operation::is_done(inner),
                        [<$op Inner>]::V11(inner) => Operation::is_done(inner),
                        [<$op Inner>]::V12(inner) => Operation::is_done(inner),
                        [<$op Inner>]::V12_1(inner) => Operation::is_done(inner),
                    }
                }
            }

            impl OperationGenerator<'_,> for  $gen {
                type O1 = [<$op Op>];
                type O2 = crate::firmware::driver::NullOp;

                fn generate(
                    &mut self,
                    device: &Device,
                    version: Version,
                ) -> Option<(Self::O1, Self::O2)> {
                    Some((
                        [<$op Op>] {
                            inner: match version {
                                Version::V10 => [<$op Inner>]::V10(
                                    crate::firmware::v10::operation::OperationGenerator::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                                Version::V11 => [<$op Inner>]::V11(
                                    crate::firmware::v11::operation::OperationGenerator::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                                Version::V12 => [<$op Inner>]::V12(
                                    crate::firmware::v12::operation::OperationGenerator::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                                Version::V12_1 => [<$op Inner>]::V12_1(
                                    crate::firmware::v12_1::operation::OperationGenerator::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                            },
                        },
                        crate::firmware::driver::NullOp,
                    ))
                }
            }
        }
    };
    ($op:ty) => {
        impl_auto_op!($op, $op);
    };
    ($($generics:ty),*;$op:ty,$gen:ty) => {
        paste::paste! {
            enum [<$op Inner>] {
                V10(crate::firmware::v10::operation::[<$op Op>]),
                V11(crate::firmware::v11::operation::[<$op Op>]),
                V12(crate::firmware::v12::operation::[<$op Op>]),
                V12_1(crate::firmware::v12_1::operation::[<$op Op>]),
            }

            #[doc(hidden)]
            pub struct [<$op Op>] {
                inner: [<$op Inner>] ,
            }

            impl Operation<'_,> for  [<$op Op>] {
                type Error = crate::error::AUTDDriverError;

                fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
                    Ok(match &mut self.inner {
                        [<$op Inner>]::V10(inner) => Operation::pack(inner, device, tx)?,
                        [<$op Inner>]::V11(inner) => Operation::pack(inner, device, tx)?,
                        [<$op Inner>]::V12(inner) => Operation::pack(inner, device, tx)?,
                        [<$op Inner>]::V12_1(inner) => Operation::pack(inner, device, tx)?,
                    })
                }

                fn required_size(&self, device: &Device) -> usize {
                    match &self.inner {
                        [<$op Inner>]::V10(inner) => Operation::required_size(inner, device),
                        [<$op Inner>]::V11(inner) => Operation::required_size(inner, device),
                        [<$op Inner>]::V12(inner) => Operation::required_size(inner, device),
                        [<$op Inner>]::V12_1(inner) => Operation::required_size(inner, device),
                    }
                }

                fn is_done(&self) -> bool {
                    match &self.inner {
                        [<$op Inner>]::V10(inner) => Operation::is_done(inner),
                        [<$op Inner>]::V11(inner) => Operation::is_done(inner),
                        [<$op Inner>]::V12(inner) => Operation::is_done(inner),
                        [<$op Inner>]::V12_1(inner) => Operation::is_done(inner),
                    }
                }
            }

            impl<'dev, $($generics),*> OperationGenerator<'dev> for  $gen
            where
                Self: crate::firmware::v10::operation::OperationGenerator<'dev,
                        O1 = crate::firmware::v10::operation::[<$op Op>],
                    > + crate::firmware::v11::operation::OperationGenerator<'dev,
                        O1 = crate::firmware::v11::operation::[<$op Op>],
                    > + crate::firmware::v12::operation::OperationGenerator<'dev,
                        O1 = crate::firmware::v12::operation::[<$op Op>],
                    > + crate::firmware::v12_1::operation::OperationGenerator<'dev,
                        O1 = crate::firmware::v12_1::operation::[<$op Op>],
                    >,
            {
                type O1 = [<$op Op>];
                type O2 = crate::firmware::driver::NullOp;

                fn generate(&mut self, device: &'dev Device ,version: Version) -> Option<(Self::O1, Self::O2)> {
                    Some((
                        Self::O1 {
                            inner: match version {
                                Version::V10 => [<$op Inner>]::V10(
                                    <Self as crate::firmware::v10::operation::OperationGenerator>::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                                Version::V11 => [<$op Inner>]::V11(
                                    crate::firmware::v11::operation::OperationGenerator::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                                Version::V12 => [<$op Inner>]::V12(
                                    crate::firmware::v12::operation::OperationGenerator::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                                Version::V12_1 => [<$op Inner>]::V12_1(
                                    crate::firmware::v12_1::operation::OperationGenerator::generate(
                                        self, device
                                    )?
                                    .0,
                                ),
                            },
                        },
                        crate::firmware::driver::NullOp,
                    ))
                }
            }
        }
    };
}

use crate::datagram::{
    Clear, CpuGPIOOutputs, EmulateGPIOIn, FetchFirmwareInfoOpGenerator, FixedCompletionSteps,
    FixedCompletionTime, FixedUpdateRate, ForceFan, GPIOOutputs, ReadsFPGAState, SwapSegment,
    Synchronize,
};
use autd3_core::modulation::ModulationOperationGenerator;

impl_auto_op!(FixedCompletionSteps);
impl_auto_op!(FixedCompletionTime);
impl_auto_op!(FixedUpdateRate);
impl_auto_op!(Clear);
impl_auto_op!(Synchronize);
impl_auto_op!(SwapSegment);
impl_auto_op!(FirmInfo, FetchFirmwareInfoOpGenerator);
impl_auto_op!(Modulation, ModulationOperationGenerator);
impl_auto_op!(F; ForceFan, ForceFan<F>);
impl_auto_op!(F; CpuGPIOOutputs, CpuGPIOOutputs<F>);
impl_auto_op!(F; EmulateGPIOIn, EmulateGPIOIn<F>);
impl_auto_op!(F; GPIOOutputs, GPIOOutputs<F>);
impl_auto_op!(F; ReadsFPGAState, ReadsFPGAState<F>);
