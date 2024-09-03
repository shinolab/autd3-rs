use std::mem::size_of;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Drive, Segment},
        operation::{write_to_tx, Operation, TypeTag},
    },
    geometry::{Device, Transducer},
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GainControlFlags(u8);

bitflags::bitflags! {
    impl GainControlFlags : u8 {
        const NONE   = 0;
        const UPDATE = 1 << 0;
    }
}

#[repr(C, align(2))]
struct GainT {
    tag: TypeTag,
    segment: u8,
    flag: GainControlFlags,
    __pad: u8,
}

pub struct GainOp {
    gain: Box<dyn Fn(&Transducer) -> Drive + Sync + Send>,
    is_done: bool,
    segment: Segment,
    transition: bool,
}

impl GainOp {
    pub const fn new(
        segment: Segment,
        transition: bool,
        gain: Box<dyn Fn(&Transducer) -> Drive + Sync + Send>,
    ) -> Self {
        Self {
            gain,
            is_done: false,
            segment,
            transition,
        }
    }
}

impl Operation for GainOp {
    fn required_size(&self, device: &Device) -> usize {
        size_of::<GainT>() + device.num_transducers() * size_of::<Drive>()
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        unsafe {
            write_to_tx(
                GainT {
                    tag: TypeTag::Gain,
                    segment: self.segment as u8,
                    flag: if self.transition {
                        GainControlFlags::UPDATE
                    } else {
                        GainControlFlags::NONE
                    },
                    __pad: 0,
                },
                tx,
            );
            device.iter().enumerate().for_each(|(i, tr)| {
                write_to_tx(
                    (self.gain)(tr),
                    &mut tx[size_of::<GainT>() + i * size_of::<Drive>()..],
                );
            });
        }

        self.is_done = true;
        Ok(size_of::<GainT>() + device.len() * size_of::<Drive>())
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use size_of;

    use rand::prelude::*;

    use super::*;
    use crate::{
        firmware::fpga::{EmitIntensity, Phase},
        geometry::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

        let mut tx = vec![0x00u8; size_of::<GainT>() + NUM_TRANS_IN_UNIT * size_of::<Drive>()];

        let mut rng = rand::thread_rng();
        let data: Vec<_> = (0..NUM_TRANS_IN_UNIT)
            .map(|_| {
                Drive::new(
                    Phase::new(rng.gen_range(0x00..=0xFF)),
                    EmitIntensity::new(rng.gen_range(0..=0xFF)),
                )
            })
            .collect();

        let mut op = GainOp::new(Segment::S0, true, {
            let data = data.clone();
            Box::new(move |tr| data[tr.local_idx()])
        });

        assert_eq!(
            op.required_size(&device),
            size_of::<GainT>() + NUM_TRANS_IN_UNIT * size_of::<Drive>()
        );

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Gain as u8);
        tx.iter()
            .skip(size_of::<GainT>())
            .collect::<Vec<_>>()
            .chunks(2)
            .zip(data.iter())
            .for_each(|(d, g)| {
                assert_eq!(d[0], &g.phase().value());
                assert_eq!(d[1], &g.intensity().value());
            });
    }
}
