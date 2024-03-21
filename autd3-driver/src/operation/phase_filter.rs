use std::collections::HashMap;

use crate::{
    common::Phase,
    derive::Transducer,
    error::AUTDInternalError,
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

#[repr(C, align(2))]
struct PhaseFilter {
    tag: TypeTag,
}

pub struct ConfigurePhaseFilterOp<F: Fn(&Device, &Transducer) -> Phase> {
    remains: HashMap<usize, usize>,
    f: F,
}

impl<F: Fn(&Device, &Transducer) -> Phase> ConfigurePhaseFilterOp<F> {
    pub fn new(f: F) -> Self {
        Self {
            remains: Default::default(),
            f,
        }
    }
}

impl<F: Fn(&Device, &Transducer) -> Phase> Operation for ConfigurePhaseFilterOp<F> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        let d = cast::<PhaseFilter>(tx);
        d.tag = TypeTag::PhaseFilter;

        unsafe {
            std::slice::from_raw_parts_mut(
                tx[std::mem::size_of::<PhaseFilter>()..].as_mut_ptr() as *mut Phase,
                device.num_transducers(),
            )
            .iter_mut()
            .zip(device.iter())
            .for_each(|(d, s)| *d = (self.f)(device, s));
        }

        Ok(std::mem::size_of::<PhaseFilter>()
            + (((device.num_transducers() + 1) >> 1) << 1) * std::mem::size_of::<Phase>())
    }

    fn required_size(&self, device: &Device) -> usize {
        std::mem::size_of::<PhaseFilter>()
            + (((device.num_transducers() + 1) >> 1) << 1) * std::mem::size_of::<Phase>()
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

        let mut tx = [0x00u8;
            (std::mem::size_of::<PhaseFilter>()
                + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>())
                * NUM_DEVICE];

        let mut op =
            ConfigurePhaseFilterOp::new(|dev, tr| Phase::new((dev.idx() + tr.idx()) as u8));

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                std::mem::size_of::<PhaseFilter>()
                    + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>(),
                op.required_size(dev)
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 1));

        geometry.devices().for_each(|dev| {
            assert!(op
                .pack(
                    dev,
                    &mut tx[dev.idx()
                        * (std::mem::size_of::<PhaseFilter>()
                            + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>())..]
                )
                .is_ok());
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                tx[dev.idx()
                    * (std::mem::size_of::<PhaseFilter>()
                        + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>())],
                TypeTag::PhaseFilter as u8
            );
            (0..dev.num_transducers()).for_each(|i| {
                assert_eq!(
                    tx[dev.idx()
                        * (std::mem::size_of::<PhaseFilter>()
                            + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>())
                        + std::mem::size_of::<PhaseFilter>()
                        + i],
                    (dev.idx() + i) as u8
                );
            });
        });
    }
}
