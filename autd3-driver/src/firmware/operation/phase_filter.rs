use crate::{
    derive::Transducer,
    error::AUTDInternalError,
    firmware::{
        fpga::Phase,
        operation::{cast, Operation, TypeTag},
    },
    geometry::{Device, Geometry},
};

use super::Remains;

#[repr(C, align(2))]
struct PhaseFilter {
    tag: TypeTag,
}

pub struct ConfigurePhaseFilterOp<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> {
    remains: Remains,
    f: F,
}

impl<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> ConfigurePhaseFilterOp<FT, F> {
    pub fn new(f: F) -> Self {
        Self {
            remains: Default::default(),
            f,
        }
    }
}

impl<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> Operation
    for ConfigurePhaseFilterOp<FT, F>
{
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        cast::<PhaseFilter>(tx).tag = TypeTag::PhaseFilter;

        let f = (self.f)(device);
        unsafe {
            std::slice::from_raw_parts_mut(
                tx[std::mem::size_of::<PhaseFilter>()..].as_mut_ptr() as *mut Phase,
                device.num_transducers(),
            )
            .iter_mut()
            .zip(device.iter())
            .for_each(|(d, s)| *d = f(s));
        }

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<PhaseFilter>()
            + (((device.num_transducers() + 1) >> 1) << 1) * std::mem::size_of::<Phase>())
    }

    fn required_size(&self, device: &Device) -> usize {
        std::mem::size_of::<PhaseFilter>()
            + (((device.num_transducers() + 1) >> 1) << 1) * std::mem::size_of::<Phase>()
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
    use crate::{defined::FREQ_40K, geometry::tests::create_geometry};

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut tx = [0x00u8;
            (std::mem::size_of::<PhaseFilter>()
                + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>())
                * NUM_DEVICE];

        let mut op = ConfigurePhaseFilterOp::new(|dev| {
            let dev_idx = dev.idx();
            move |tr| Phase::new((dev_idx + tr.idx()) as u8)
        });

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
            .for_each(|dev| assert_eq!(op.remains[dev], 1));

        geometry.devices().for_each(|dev| {
            assert!(op
                .pack(
                    dev,
                    &mut tx[dev.idx()
                        * (std::mem::size_of::<PhaseFilter>()
                            + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>())..]
                )
                .is_ok());
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

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
