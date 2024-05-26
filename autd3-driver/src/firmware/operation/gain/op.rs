use super::GainControlFlags;
use crate::{
    derive::Transducer,
    error::AUTDInternalError,
    firmware::{
        fpga::{Drive, Segment},
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

#[repr(C, align(2))]
struct GainT {
    tag: TypeTag,
    segment: u8,
    flag: GainControlFlags,
    __pad: u8,
}

pub struct GainOp<'a> {
    gain: Box<dyn Fn(&Transducer) -> Drive + 'a>,
    is_done: bool,
    segment: Segment,
    transition: bool,
}

impl<'a> GainOp<'a> {
    pub fn new(
        segment: Segment,
        transition: bool,
        gain: Box<dyn Fn(&Transducer) -> Drive + 'a>,
    ) -> Self {
        Self {
            gain,
            is_done: false,
            segment,
            transition,
        }
    }
}

impl<'a> Operation for GainOp<'a> {
    fn required_size(&self, device: &Device) -> usize {
        std::mem::size_of::<GainT>() + device.num_transducers() * std::mem::size_of::<Drive>()
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<GainT>(tx) = GainT {
            tag: TypeTag::Gain,
            segment: self.segment as u8,
            flag: GainControlFlags::NONE,
            __pad: 0,
        };
        cast::<GainT>(tx)
            .flag
            .set(GainControlFlags::UPDATE, self.transition);

        unsafe {
            let dst = std::slice::from_raw_parts_mut(
                tx[std::mem::size_of::<GainT>()..].as_mut_ptr() as *mut Drive,
                device.len(),
            );
            dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                *d = (self.gain)(tr);
            });
        }

        self.is_done = true;
        Ok(std::mem::size_of::<GainT>() + device.len() * std::mem::size_of::<Drive>())
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use rand::prelude::*;

    use super::*;
    use crate::{
        firmware::fpga::{EmitIntensity, Phase},
        geometry::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

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
            Box::new(move |tr| data[tr.idx()])
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
            .skip(std::mem::size_of::<GainT>())
            .collect::<Vec<_>>()
            .chunks(2)
            .zip(data.iter())
            .for_each(|(d, g)| {
                assert_eq!(d[0], &g.phase().value());
                assert_eq!(d[1], &g.intensity().value());
            });
    }
}
