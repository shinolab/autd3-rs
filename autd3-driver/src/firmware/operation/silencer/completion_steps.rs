use super::{SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS, SILENCER_CTL_FLAG_STRICT_MODE};
use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::Device,
};

#[repr(C, align(2))]
struct SilencerFixedCompletionSteps {
    tag: TypeTag,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

pub struct SilencerFixedCompletionStepsOp {
    is_done: bool,
    value_intensity: u16,
    value_phase: u16,
    strict_mode: bool,
}

impl SilencerFixedCompletionStepsOp {
    pub fn new(value_intensity: u16, value_phase: u16, strict_mode: bool) -> Self {
        Self {
            is_done: false,
            value_intensity,
            value_phase,
            strict_mode,
        }
    }
}

impl Operation for SilencerFixedCompletionStepsOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<SilencerFixedCompletionSteps>(tx) = SilencerFixedCompletionSteps {
            tag: TypeTag::Silencer,
            flag: SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS
                | if self.strict_mode {
                    SILENCER_CTL_FLAG_STRICT_MODE
                } else {
                    0
                },
            value_intensity: self.value_intensity,
            value_phase: self.value_phase,
        };

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
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[rstest::rstest]
    #[test]
    #[case(SILENCER_CTL_FLAG_STRICT_MODE, true)]
    #[case(0x00, false)]
    fn test(#[case] value: u8, #[case] strict_mode: bool) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedCompletionSteps>()];

        let mut op = SilencerFixedCompletionStepsOp::new(0x1234, 0x5678, strict_mode);

        assert_eq!(
            op.required_size(&device),
            size_of::<SilencerFixedCompletionSteps>()
        );
        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Silencer as u8);
        assert_eq!(tx[1], value);
        assert_eq!(tx[2], 0x34);
        assert_eq!(tx[3], 0x12);
        assert_eq!(tx[4], 0x78);
        assert_eq!(tx[5], 0x56);
    }
}
