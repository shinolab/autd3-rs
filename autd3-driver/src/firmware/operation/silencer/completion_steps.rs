use super::{SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS, SILENCER_CTL_FLAG_STRICT_MODE};
use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, Remains, TypeTag},
    geometry::{Device, Geometry},
};

#[repr(C, align(2))]
struct ConfigSilencerFixedCompletionSteps {
    tag: TypeTag,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

pub struct ConfigSilencerFixedCompletionStepsOp {
    remains: Remains,
    value_intensity: u16,
    value_phase: u16,
    strict_mode: bool,
}

impl ConfigSilencerFixedCompletionStepsOp {
    pub fn new(value_intensity: u16, value_phase: u16, strict_mode: bool) -> Self {
        Self {
            remains: Default::default(),
            value_intensity,
            value_phase,
            strict_mode,
        }
    }
}

impl Operation for ConfigSilencerFixedCompletionStepsOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<ConfigSilencerFixedCompletionSteps>(tx) = ConfigSilencerFixedCompletionSteps {
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

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<ConfigSilencerFixedCompletionSteps>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ConfigSilencerFixedCompletionSteps>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, |_| 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::tests::create_geometry;

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 8 * NUM_DEVICE];

        let mut op = ConfigSilencerFixedCompletionStepsOp::new(0x1234, 0x5678, false);

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 6));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 6..]).is_ok());
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 6], TypeTag::Silencer as u8);
            assert_eq!(tx[dev.idx() * 6 + 1], 0);
            assert_eq!(tx[dev.idx() * 6 + 2], 0x34);
            assert_eq!(tx[dev.idx() * 6 + 3], 0x12);
            assert_eq!(tx[dev.idx() * 6 + 4], 0x78);
            assert_eq!(tx[dev.idx() * 6 + 5], 0x56);
        });
    }

    #[test]
    fn test_with_strict_mode() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 8 * NUM_DEVICE];

        let mut op = ConfigSilencerFixedCompletionStepsOp::new(0x1234, 0x5678, true);

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 6));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 6..]).is_ok());
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 6], TypeTag::Silencer as u8);
            assert_eq!(tx[dev.idx() * 6 + 1], SILENCER_CTL_FLAG_STRICT_MODE);
            assert_eq!(tx[dev.idx() * 6 + 2], 0x34);
            assert_eq!(tx[dev.idx() * 6 + 3], 0x12);
            assert_eq!(tx[dev.idx() * 6 + 4], 0x78);
            assert_eq!(tx[dev.idx() * 6 + 5], 0x56);
        });
    }
}
