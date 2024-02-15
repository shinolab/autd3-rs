use std::collections::HashMap;

use crate::{
    common::Drive,
    cpu::Segment,
    datagram::{Gain, GainFilter},
    error::AUTDInternalError,
    fpga::FPGADrive,
    geometry::{Device, Geometry},
    operation::{cast, TypeTag},
};

use super::Operation;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GainControlFlags(u16);

bitflags::bitflags! {
    impl GainControlFlags : u16 {
        const NONE           = 0;
        const UPDATE_SEGMENT = 1 << 0;
    }
}

#[repr(C, align(2))]
struct GainT {
    tag: TypeTag,
    segment: u8,
    flag: GainControlFlags,
}

#[repr(C, align(2))]
struct GainUpdate {
    tag: TypeTag,
    segment: u8,
}

pub struct GainOp<G: Gain> {
    gain: G,
    drives: HashMap<usize, Vec<Drive>>,
    remains: HashMap<usize, usize>,
    segment: Segment,
    update_segment: bool,
}

impl<G: Gain> GainOp<G> {
    pub fn new(segment: Segment, update_segment: bool, gain: G) -> Self {
        Self {
            gain,
            drives: Default::default(),
            remains: Default::default(),
            segment,
            update_segment,
        }
    }
}

impl<G: Gain> Operation for GainOp<G> {
    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.drives = self.gain.calc(geometry, GainFilter::All)?;
        self.remains = geometry.devices().map(|device| (device.idx(), 1)).collect();
        Ok(())
    }

    fn required_size(&self, device: &Device) -> usize {
        std::mem::size_of::<GainT>() + device.num_transducers() * std::mem::size_of::<FPGADrive>()
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        let d = &self.drives[&device.idx()];
        assert!(
            tx.len() >= std::mem::size_of::<GainT>() + d.len() * std::mem::size_of::<FPGADrive>()
        );

        cast::<GainT>(tx).tag = TypeTag::Gain;
        cast::<GainT>(tx).segment = self.segment as u8;
        cast::<GainT>(tx).flag = GainControlFlags::NONE;
        cast::<GainT>(tx)
            .flag
            .set(GainControlFlags::UPDATE_SEGMENT, self.update_segment);

        unsafe {
            std::slice::from_raw_parts_mut(
                tx[std::mem::size_of::<GainT>()..].as_mut_ptr() as *mut FPGADrive,
                d.len(),
            )
            .iter_mut()
            .zip(d.iter())
            .for_each(|(d, s)| d.set(s));
        }

        Ok(std::mem::size_of::<GainT>() + d.len() * std::mem::size_of::<FPGADrive>())
    }

    fn commit(&mut self, device: &Device) {
        self.remains
            .insert(device.idx(), self.remains[&device.idx()] - 1);
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }
}

pub struct GainChangeSegmentOp {
    segment: Segment,
    remains: HashMap<usize, usize>,
}

impl GainChangeSegmentOp {
    pub fn new(segment: Segment) -> Self {
        Self {
            segment,
            remains: HashMap::new(),
        }
    }
}

impl Operation for GainChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        let d = cast::<GainUpdate>(tx);
        d.tag = TypeTag::GainChangeSegment;
        d.segment = self.segment as u8;

        Ok(std::mem::size_of::<GainUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<GainUpdate>()
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
    use rand::prelude::*;

    use super::*;
    use crate::{
        common::{EmitIntensity, Phase},
        geometry::tests::create_geometry,
        operation::tests::{ErrGain, TestGain},
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn gain_op() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![
            0x00u8;
            (std::mem::size_of::<GainT>()
                + NUM_TRANS_IN_UNIT * std::mem::size_of::<FPGADrive>())
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
                std::mem::size_of::<GainT>() + NUM_TRANS_IN_UNIT * std::mem::size_of::<FPGADrive>()
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
                        * (std::mem::size_of::<GainT>()
                            + NUM_TRANS_IN_UNIT * std::mem::size_of::<FPGADrive>())..]
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
                    * (std::mem::size_of::<GainT>()
                        + NUM_TRANS_IN_UNIT * std::mem::size_of::<FPGADrive>())],
                TypeTag::Gain as u8
            );
            tx.iter()
                .skip(
                    dev.idx()
                        * (std::mem::size_of::<GainT>()
                            + NUM_TRANS_IN_UNIT * std::mem::size_of::<FPGADrive>())
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
    fn error_gain() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let gain = ErrGain {
            segment: Segment::S0,
            update_segment: true,
        };
        let mut op = GainOp::<ErrGain>::new(Segment::S0, true, gain);

        assert_eq!(
            op.init(&geometry),
            Err(AUTDInternalError::GainError("test".to_owned()))
        );
    }
}
