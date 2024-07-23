use std::num::NonZeroU8;

use crate::{
    error::AUTDInternalError,
    firmware::operation::{write_to_tx, Operation, TypeTag},
    geometry::Device,
};

use super::{SILENCER_FLAG_FIXED_UPDATE_RATE_MODE, SILENCER_FLAG_PULSE_WIDTH};

#[repr(C, align(2))]
struct SilencerFixedUpdateRate {
    tag: TypeTag,
    flag: u8,
    value_intensity: u8,
    value_phase: u8,
}

pub struct SilencerFixedUpdateRateOp {
    is_done: bool,
    value_intensity: NonZeroU8,
    value_phase: NonZeroU8,
    target: super::SilencerTarget,
}

impl SilencerFixedUpdateRateOp {
    pub const fn new(
        value_intensity: NonZeroU8,
        value_phase: NonZeroU8,
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
        write_to_tx(
            SilencerFixedUpdateRate {
                tag: TypeTag::Silencer,
                flag: SILENCER_FLAG_FIXED_UPDATE_RATE_MODE
                    | match self.target {
                        super::SilencerTarget::Intensity => 0,
                        super::SilencerTarget::PulseWidth => SILENCER_FLAG_PULSE_WIDTH,
                    },
                value_intensity: self.value_intensity.get(),
                value_phase: self.value_phase.get(),
            },
            tx,
        );

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

        let mut op = SilencerFixedUpdateRateOp::new(
            NonZeroU8::new(0x12).unwrap(),
            NonZeroU8::new(0x34).unwrap(),
            SilencerTarget::Intensity,
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<SilencerFixedUpdateRate>()
        );
        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Silencer as u8);
        assert_eq!(tx[1], SILENCER_FLAG_FIXED_UPDATE_RATE_MODE);
        assert_eq!(tx[2], 0x12);
        assert_eq!(tx[3], 0x34);
    }
}
