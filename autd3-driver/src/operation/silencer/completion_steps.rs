use std::collections::HashMap;

use super::{SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS, SILENCER_CTL_FLAG_STRICT_MODE};
use crate::{
    error::AUTDInternalError,
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

#[repr(C, align(2))]
struct ConfigSilencerFixedCompletionSteps {
    tag: TypeTag,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

pub struct ConfigSilencerFixedCompletionStepsOp {
    remains: HashMap<usize, usize>,
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
        assert_eq!(self.remains[&device.idx()], 1);

        let d = cast::<ConfigSilencerFixedCompletionSteps>(tx);
        d.tag = TypeTag::Silencer;
        d.value_intensity = self.value_intensity;
        d.value_phase = self.value_phase;
        d.flag = SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS;
        if self.strict_mode {
            d.flag |= SILENCER_CTL_FLAG_STRICT_MODE;
        }

        Ok(std::mem::size_of::<ConfigSilencerFixedCompletionSteps>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ConfigSilencerFixedCompletionSteps>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains = geometry.devices().map(|device| (device.idx(), 1)).collect();
        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }

    fn commit(&mut self, device: &Device) {
        self.remains.insert(device.idx(), 0);
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
            .for_each(|dev| assert_eq!(op.remains(dev), 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 6..]).is_ok());
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

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
            .for_each(|dev| assert_eq!(op.remains(dev), 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 6..]).is_ok());
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

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
