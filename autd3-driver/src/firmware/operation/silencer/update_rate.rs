use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{SILENCER_VALUE_MAX, SILENCER_VALUE_MIN},
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

use super::{SILENCER_FLAG_FIXED_UPDATE_RATE_MODE, SILENCER_FLAG_PULSE_WIDTH};

#[repr(C, align(2))]
struct SilencerFixedUpdateRate {
    tag: TypeTag,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

pub struct SilencerFixedUpdateRateOp {
    is_done: bool,
    value_intensity: u16,
    value_phase: u16,
    target: super::SilencerTarget,
}

impl SilencerFixedUpdateRateOp {
    pub const fn new(
        value_intensity: u16,
        value_phase: u16,
        target: super::SilencerTarget,
    ) -> Self {
        Self {
            is_done: false,
            value_intensity,
            value_phase,
            target,
        }
    }
}

impl Operation for SilencerFixedUpdateRateOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        if !(SILENCER_VALUE_MIN..=SILENCER_VALUE_MAX).contains(&self.value_intensity) {
            return Err(AUTDInternalError::SilencerUpdateRateOutOfRange(
                self.value_intensity,
            ));
        }
        if !(SILENCER_VALUE_MIN..=SILENCER_VALUE_MAX).contains(&self.value_phase) {
            return Err(AUTDInternalError::SilencerUpdateRateOutOfRange(
                self.value_phase,
            ));
        }

        *cast::<SilencerFixedUpdateRate>(tx) = SilencerFixedUpdateRate {
            tag: TypeTag::Silencer,
            flag: SILENCER_FLAG_FIXED_UPDATE_RATE_MODE
                | match self.target {
                    super::SilencerTarget::Intensity => 0,
                    super::SilencerTarget::PulseWidth => SILENCER_FLAG_PULSE_WIDTH,
                },
            value_intensity: self.value_intensity,
            value_phase: self.value_phase,
        };

        self.is_done = true;
        Ok(std::mem::size_of::<SilencerFixedUpdateRate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<SilencerFixedUpdateRate>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::firmware::operation::SilencerTarget;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedUpdateRate>()];

        let mut op = SilencerFixedUpdateRateOp::new(0x1234, 0x5678, SilencerTarget::Intensity);

        assert_eq!(
            op.required_size(&device),
            size_of::<SilencerFixedUpdateRate>()
        );
        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Silencer as u8);
        assert_eq!(tx[1], SILENCER_FLAG_FIXED_UPDATE_RATE_MODE);
        assert_eq!(tx[2], 0x34);
        assert_eq!(tx[3], 0x12);
        assert_eq!(tx[4], 0x78);
        assert_eq!(tx[5], 0x56);
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(size_of::<SilencerFixedUpdateRate>()), SILENCER_VALUE_MIN, SILENCER_VALUE_MIN)]
    #[case(Ok(size_of::<SilencerFixedUpdateRate>()), SILENCER_VALUE_MAX, SILENCER_VALUE_MAX)]
    #[case(Err(AUTDInternalError::SilencerUpdateRateOutOfRange(0)), SILENCER_VALUE_MAX, SILENCER_VALUE_MIN - 1)]
    #[case(Err(AUTDInternalError::SilencerUpdateRateOutOfRange(0)), SILENCER_VALUE_MIN - 1, SILENCER_VALUE_MAX)]
    fn out_of_range(
        #[case] expected: Result<usize, AUTDInternalError>,
        #[case] value_intensity: u16,
        #[case] value_phase: u16,
    ) {
        const FRAME_SIZE: usize = size_of::<SilencerFixedUpdateRate>() + NUM_TRANS_IN_UNIT * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op =
            SilencerFixedUpdateRateOp::new(value_intensity, value_phase, SilencerTarget::Intensity);

        assert_eq!(expected, op.pack(&device, &mut tx));
    }
}
