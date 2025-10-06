use std::{convert::Infallible, num::NonZeroU16};

use super::SilencerControlFlags;
use crate::{
    datagram::FixedCompletionSteps,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::geometry::Device;

#[repr(C, align(2))]
struct SilencerFixedCompletionSteps {
    tag: TypeTag,
    flag: SilencerControlFlags,
    value_intensity: u16,
    value_phase: u16,
}

pub struct FixedCompletionStepsOp {
    is_done: bool,
    intensity: NonZeroU16,
    phase: NonZeroU16,
    strict: bool,
}

impl FixedCompletionStepsOp {
    pub(crate) const fn new(intensity: NonZeroU16, phase: NonZeroU16, strict: bool) -> Self {
        Self {
            is_done: false,
            intensity,
            phase,
            strict,
        }
    }
}

impl Operation<'_> for FixedCompletionStepsOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::operation::write_to_tx(
            tx,
            SilencerFixedCompletionSteps {
                tag: TypeTag::Silencer,
                flag: if self.strict {
                    SilencerControlFlags::STRICT_MODE
                } else {
                    SilencerControlFlags::NONE
                },
                value_intensity: self.intensity.get(),
                value_phase: self.phase.get(),
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<SilencerFixedCompletionSteps>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<SilencerFixedCompletionSteps>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl OperationGenerator<'_> for FixedCompletionSteps {
    type O1 = FixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(self.intensity, self.phase, self.strict),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[rstest::rstest]
    #[case(SilencerControlFlags::STRICT_MODE.bits(), true)]
    #[case(0x00, false)]
    fn test(#[case] value: u8, #[case] strict: bool) {
        let device = crate::autd3_device::tests::create_device();

        let mut tx = [0x00u8; size_of::<SilencerFixedCompletionSteps>()];

        let mut op = FixedCompletionStepsOp::new(
            NonZeroU16::new(0x12).unwrap(),
            NonZeroU16::new(0x34).unwrap(),
            strict,
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<SilencerFixedCompletionSteps>()
        );
        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Silencer as u8);
        assert_eq!(tx[1], value);
        assert_eq!(tx[2], 0x12);
        assert_eq!(tx[3], 0x00);
        assert_eq!(tx[4], 0x34);
        assert_eq!(tx[5], 0x00);
    }
}
