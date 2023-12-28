/*
 * File: update_rate.rs
 * Project: silencer
 * Created Date: 27/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 27/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use super::{SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS, SILENCER_CTL_FLAG_STRICT_MODE};
use crate::{
    error::AUTDInternalError,
    geometry::{Device, Geometry},
    operation::{Operation, TypeTag},
};

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
        tx[0] = TypeTag::Silencer as u8;
        tx[2] = (self.value_intensity & 0xFF) as u8;
        tx[3] = (self.value_intensity >> 8) as u8;
        tx[4] = (self.value_phase & 0xFF) as u8;
        tx[5] = (self.value_phase >> 8) as u8;

        let mut flag: u16 = SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS;
        if self.strict_mode {
            flag |= SILENCER_CTL_FLAG_STRICT_MODE;
        }
        tx[6] = (flag & 0xFF) as u8;
        tx[7] = (flag >> 8) as u8;
        Ok(8)
    }

    fn required_size(&self, _: &Device) -> usize {
        8
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
    fn silencer_op_fixed_completion_steps() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 8 * NUM_DEVICE];

        let mut op = ConfigSilencerFixedCompletionStepsOp::new(0x1234, 0x5678, false);

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 8));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 8..]).is_ok());
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 8], TypeTag::Silencer as u8);
            assert_eq!(tx[dev.idx() * 8 + 1], 0);
            assert_eq!(tx[dev.idx() * 8 + 2], 0x34);
            assert_eq!(tx[dev.idx() * 8 + 3], 0x12);
            assert_eq!(tx[dev.idx() * 8 + 4], 0x78);
            assert_eq!(tx[dev.idx() * 8 + 5], 0x56);
            assert_eq!(tx[dev.idx() * 8 + 6], 0x01);
            assert_eq!(tx[dev.idx() * 8 + 7], 0x00);
        });
    }

    #[test]
    fn silencer_op_fixed_completion_steps_strict_mode() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 8 * NUM_DEVICE];

        let mut op = ConfigSilencerFixedCompletionStepsOp::new(0x1234, 0x5678, true);

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 8));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 8..]).is_ok());
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 8], TypeTag::Silencer as u8);
            assert_eq!(tx[dev.idx() * 8 + 1], 0);
            assert_eq!(tx[dev.idx() * 8 + 2], 0x34);
            assert_eq!(tx[dev.idx() * 8 + 3], 0x12);
            assert_eq!(tx[dev.idx() * 8 + 4], 0x78);
            assert_eq!(tx[dev.idx() * 8 + 5], 0x56);
            assert_eq!(tx[dev.idx() * 8 + 6], 0x01);
            assert_eq!(tx[dev.idx() * 8 + 7], 0x01);
        });
    }
}
