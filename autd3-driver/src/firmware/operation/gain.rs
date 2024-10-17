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

pub trait GainContext: Send + Sync {
    fn calc(&self, tr: &Transducer) -> Drive;
}

pub struct GainOp<Context: GainContext> {
    context: Context,
    is_done: bool,
    segment: Segment,
    transition: bool,
}

impl<Context: GainContext> GainOp<Context> {
    pub const fn new(segment: Segment, transition: bool, context: Context) -> Self {
        Self {
            context,
            is_done: false,
            segment,
            transition,
        }
    }
}

impl<Context: GainContext> Operation for GainOp<Context> {
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
                    self.context.calc(tr),
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

        struct Context {
            data: Vec<Drive>,
        }

        impl GainContext for Context {
            fn calc(&self, tr: &Transducer) -> Drive {
                self.data[tr.idx()]
            }
        }

        let mut op = GainOp::new(Segment::S0, true, {
            let data = data.clone();
            Context { data }
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
