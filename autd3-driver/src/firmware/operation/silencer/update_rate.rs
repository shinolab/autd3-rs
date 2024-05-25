use crate::{
    error::AUTDInternalError,
    firmware::operation::{
        cast, silencer::SILENCER_CTL_FLAG_FIXED_UPDATE_RATE, Operation, TypeTag,
    },
    geometry::Device,
};

#[repr(C, align(2))]
struct ConfigSilencerFixedUpdateRate {
    tag: TypeTag,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

pub struct ConfigSilencerFixedUpdateRateOp {
    is_done: bool,
    value_intensity: u16,
    value_phase: u16,
}

impl ConfigSilencerFixedUpdateRateOp {
    pub fn new(value_intensity: u16, value_phase: u16) -> Self {
        Self {
            is_done: false,
            value_intensity,
            value_phase,
        }
    }
}

impl Operation for ConfigSilencerFixedUpdateRateOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<ConfigSilencerFixedUpdateRate>(tx) = ConfigSilencerFixedUpdateRate {
            tag: TypeTag::Silencer,
            flag: SILENCER_CTL_FLAG_FIXED_UPDATE_RATE,
            value_intensity: self.value_intensity,
            value_phase: self.value_phase,
        };

        self.is_done = true;
        Ok(std::mem::size_of::<ConfigSilencerFixedUpdateRate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ConfigSilencerFixedUpdateRate>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::{
        firmware::operation::silencer::SILNCER_MODE_FIXED_UPDATE_RATE,
        geometry::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<ConfigSilencerFixedUpdateRate>()];

        let mut op = ConfigSilencerFixedUpdateRateOp::new(0x1234, 0x5678);

        assert_eq!(
            op.required_size(&device),
            size_of::<ConfigSilencerFixedUpdateRate>()
        );
        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Silencer as u8);
        assert_eq!(tx[1], SILNCER_MODE_FIXED_UPDATE_RATE);
        assert_eq!(tx[2], 0x34);
        assert_eq!(tx[3], 0x12);
        assert_eq!(tx[4], 0x78);
        assert_eq!(tx[5], 0x56);
    }
}
