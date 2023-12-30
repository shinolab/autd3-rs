/*
 * File: debug.rs
 * Project: operation
 * Created Date: 21/11/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use crate::{
    derive::prelude::Transducer,
    error::AUTDInternalError,
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

#[repr(C, align(2))]
struct DebugOutIdx {
    tag: TypeTag,
    idx: u8,
}

pub struct DebugOutIdxOp<F: Fn(&Device) -> Option<&Transducer>> {
    remains: HashMap<usize, usize>,
    f: F,
}

impl<F: Fn(&Device) -> Option<&Transducer>> DebugOutIdxOp<F> {
    pub fn new(f: F) -> Self {
        Self {
            remains: Default::default(),
            f,
        }
    }
}

impl<F: Fn(&Device) -> Option<&Transducer>> Operation for DebugOutIdxOp<F> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        let d = cast::<DebugOutIdx>(tx);
        d.tag = TypeTag::Debug;
        d.idx = (self.f)(device).map(|tr| tr.idx() as u8).unwrap_or(0xFF);

        Ok(std::mem::size_of::<DebugOutIdx>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<DebugOutIdx>()
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
    fn debug_op() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 2 * NUM_DEVICE];

        let mut op = DebugOutIdxOp::new(|dev| Some(&dev[10]));

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 2));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::Debug as u8);
            assert_eq!(tx[dev.idx() * 2 + 1], 10);
        });
    }
}
