use std::{convert::Infallible, num::NonZeroU16};

use crate::{
    firmware::{
        fpga::SilencerTarget,
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use super::SilencerControlFlags;

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct SilencerFixedCompletionSteps {
    tag: TypeTag,
    flag: SilencerControlFlags,
    value_intensity: u16,
    value_phase: u16,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct SilencerFixedCompletionStepsOp {
    #[new(default)]
    is_done: bool,
    intensity: NonZeroU16,
    phase: NonZeroU16,
    strict_mode: bool,
    target: SilencerTarget,
}

impl Operation for SilencerFixedCompletionStepsOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::super::write_to_tx(
            tx,
            SilencerFixedCompletionSteps {
                tag: TypeTag::Silencer,
                flag: if self.strict_mode {
                    SilencerControlFlags::STRICT_MODE
                } else {
                    SilencerControlFlags::NONE
                } | match self.target {
                    SilencerTarget::Intensity => SilencerControlFlags::NONE,
                    SilencerTarget::PulseWidth => SilencerControlFlags::PULSE_WIDTH,
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::{firmware::fpga::SilencerTarget, firmware::operation::tests::create_device};

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[rstest::rstest]
    #[test]
    #[case(SilencerControlFlags::STRICT_MODE.bits(), true)]
    #[case(0x00, false)]
    fn test(#[case] value: u8, #[case] strict_mode: bool) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedCompletionSteps>()];

        let mut op = SilencerFixedCompletionStepsOp::new(
            NonZeroU16::new(0x12).unwrap(),
            NonZeroU16::new(0x34).unwrap(),
            strict_mode,
            SilencerTarget::Intensity,
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
