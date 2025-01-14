use std::mem::size_of;

use crate::{
    error::AUTDDriverError,
    firmware::{
        fpga::{Drive, Segment, TransitionMode},
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use autd3_core::gain::GainContext;
use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[derive(Clone, Copy, IntoBytes, Immutable)]
#[repr(C)]
pub struct GainControlFlags(u8);

bitflags::bitflags! {
    impl GainControlFlags : u8 {
        const NONE   = 0;
        const UPDATE = 1 << 0;
    }
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Gain {
    tag: TypeTag,
    segment: u8,
    flag: GainControlFlags,
    __: u8,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct GainOp<Context: GainContext> {
    #[new(default)]
    is_done: bool,
    segment: Segment,
    transition: Option<TransitionMode>,
    context: Context,
}

impl<Context: GainContext> Operation for GainOp<Context> {
    type Error = AUTDDriverError;

    fn required_size(&self, device: &Device) -> usize {
        size_of::<Gain>() + device.num_transducers() * size_of::<Drive>()
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::write_to_tx(
            tx,
            Gain {
                tag: TypeTag::Gain,
                segment: self.segment as u8,
                flag: if let Some(mode) = self.transition {
                    if mode != TransitionMode::Immediate {
                        return Err(AUTDDriverError::InvalidTransitionMode);
                    } else {
                        GainControlFlags::UPDATE
                    }
                } else {
                    GainControlFlags::NONE
                },
                __: 0,
            },
        );
        tx[size_of::<Gain>()..]
            .chunks_mut(size_of::<Drive>())
            .zip(device.iter())
            .for_each(|(dst, tr)| {
                super::write_to_tx(dst, self.context.calc(tr));
            });

        self.is_done = true;
        Ok(size_of::<Gain>() + device.len() * size_of::<Drive>())
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::geometry::Transducer;

    use rand::prelude::*;

    use super::*;
    use crate::{
        firmware::fpga::{EmitIntensity, Phase},
        firmware::operation::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    struct Context {
        data: Vec<Drive>,
    }

    impl GainContext for Context {
        fn calc(&self, tr: &Transducer) -> Drive {
            self.data[tr.idx()]
        }
    }

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

        let mut tx = vec![0x00u8; size_of::<Gain>() + NUM_TRANS_IN_UNIT * size_of::<Drive>()];

        let mut rng = rand::thread_rng();
        let data: Vec<_> = (0..NUM_TRANS_IN_UNIT)
            .map(|_| {
                Drive::new(
                    Phase::new(rng.gen_range(0x00..=0xFF)),
                    EmitIntensity::new(rng.gen_range(0..=0xFF)),
                )
            })
            .collect();

        let mut op = GainOp::new(Segment::S0, Some(TransitionMode::Immediate), {
            let data = data.clone();
            Context { data }
        });

        assert_eq!(
            op.required_size(&device),
            size_of::<Gain>() + NUM_TRANS_IN_UNIT * size_of::<Drive>()
        );

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Gain as u8);
        tx.iter()
            .skip(size_of::<Gain>())
            .collect::<Vec<_>>()
            .chunks(2)
            .zip(data.iter())
            .for_each(|(d, g)| {
                assert_eq!(d[0], &g.phase().value());
                assert_eq!(d[1], &g.intensity().value());
            });
    }

    #[test]
    fn invalid_transition_mode() {
        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

        let mut tx = vec![0x00u8; size_of::<Gain>() + NUM_TRANS_IN_UNIT * size_of::<Drive>()];

        let mut op = GainOp::new(Segment::S0, Some(TransitionMode::Ext), {
            Context { data: Vec::new() }
        });

        assert_eq!(
            Some(AUTDDriverError::InvalidTransitionMode),
            op.pack(&device, &mut tx).err()
        );
    }
}
