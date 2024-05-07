use std::collections::HashMap;

use super::GainControlFlags;
use crate::{
    datagram::{Gain, GainFilter},
    error::AUTDInternalError,
    firmware::{
        fpga::{Drive, Segment},
        operation::{cast, Operation, Remains, TypeTag},
    },
    geometry::{Device, Geometry},
};

#[repr(C, align(2))]
struct GainT {
    tag: TypeTag,
    segment: u8,
    flag: GainControlFlags,
}

pub struct GainOp<G: Gain> {
    gain: G,
    drives: HashMap<usize, Vec<Drive>>,
    remains: Remains,
    segment: Segment,
    transition: bool,
}

impl<G: Gain> GainOp<G> {
    pub fn new(segment: Segment, transition: bool, gain: G) -> Self {
        Self {
            gain,
            drives: Default::default(),
            remains: Default::default(),
            segment,
            transition,
        }
    }
}

impl<G: Gain> Operation for GainOp<G> {
    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.drives = self.gain.calc(geometry, GainFilter::All)?;
        self.remains.init(geometry, |_| 1);
        Ok(())
    }

    fn required_size(&self, device: &Device) -> usize {
        std::mem::size_of::<GainT>() + device.num_transducers() * std::mem::size_of::<Drive>()
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let d = &self.drives[&device.idx()];
        assert!(tx.len() >= std::mem::size_of::<GainT>() + d.len() * std::mem::size_of::<Drive>());

        *cast::<GainT>(tx) = GainT {
            tag: TypeTag::Gain,
            segment: self.segment as u8,
            flag: GainControlFlags::NONE,
        };
        cast::<GainT>(tx)
            .flag
            .set(GainControlFlags::TRANSITION, self.transition);

        unsafe {
            std::ptr::copy_nonoverlapping(
                d.as_ptr(),
                tx[std::mem::size_of::<GainT>()..].as_mut_ptr() as *mut Drive,
                d.len(),
            );
        }

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<GainT>() + d.len() * std::mem::size_of::<Drive>())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::FREQ_40K,
        firmware::{
            fpga::{EmitIntensity, Phase},
            operation::tests::{ErrGain, TestGain},
        },
        geometry::tests::create_geometry,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut tx = vec![
            0x00u8;
            (std::mem::size_of::<GainT>()
                + NUM_TRANS_IN_UNIT * std::mem::size_of::<Drive>())
                * NUM_DEVICE
        ];

        let mut rng = rand::thread_rng();
        let data = geometry
            .devices()
            .map(|dev| {
                (
                    dev.idx(),
                    (0..dev.num_transducers())
                        .map(|_| {
                            Drive::new(
                                Phase::new(rng.gen_range(0x00..=0xFF)),
                                EmitIntensity::new(rng.gen_range(0..=0xFF)),
                            )
                        })
                        .collect(),
                )
            })
            .collect();
        let gain = TestGain { data };
        let mut op = GainOp::<TestGain>::new(Segment::S0, true, gain.clone());

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainT>() + NUM_TRANS_IN_UNIT * std::mem::size_of::<Drive>()
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
                        * (std::mem::size_of::<GainT>()
                            + NUM_TRANS_IN_UNIT * std::mem::size_of::<Drive>())..]
                )
                .is_ok());
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                tx[dev.idx()
                    * (std::mem::size_of::<GainT>()
                        + NUM_TRANS_IN_UNIT * std::mem::size_of::<Drive>())],
                TypeTag::Gain as u8
            );
            tx.iter()
                .skip(
                    dev.idx()
                        * (std::mem::size_of::<GainT>()
                            + NUM_TRANS_IN_UNIT * std::mem::size_of::<Drive>())
                        + std::mem::size_of::<GainT>(),
                )
                .collect::<Vec<_>>()
                .chunks(2)
                .zip(gain.data[&dev.idx()].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], &g.phase().value());
                    assert_eq!(d[1], &g.intensity().value());
                })
        });
    }

    #[test]
    fn test_error() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let gain = ErrGain {
            segment: Segment::S0,
            transition: true,
        };
        let mut op = GainOp::<ErrGain>::new(Segment::S0, true, gain);

        assert_eq!(
            op.init(&geometry),
            Err(AUTDInternalError::GainError("test".to_owned()))
        );
    }
}
