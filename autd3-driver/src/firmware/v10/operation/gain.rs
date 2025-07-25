use std::mem::size_of;

use super::OperationGenerator;
use crate::{
    error::AUTDDriverError,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
};

use autd3_core::{
    firmware::{Drive, Segment, transition_mode::TransitionMode},
    gain::{GainCalculator, GainCalculatorGenerator, GainOperationGenerator},
    geometry::Device,
};

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

pub struct GainOp<Calculator> {
    is_done: bool,
    segment: Segment,
    transition: bool,
    calculator: Calculator,
}

impl<'a, Calculator: GainCalculator<'a>> GainOp<Calculator> {
    pub(crate) const fn new(segment: Segment, transition: bool, calculator: Calculator) -> Self {
        Self {
            is_done: false,
            segment,
            transition,
            calculator,
        }
    }
}

impl<'a, Calculator: GainCalculator<'a>> Operation<'a> for GainOp<Calculator> {
    type Error = AUTDDriverError;

    fn required_size(&self, device: &'a Device) -> usize {
        size_of::<Gain>() + device.num_transducers() * size_of::<Drive>()
    }

    fn pack(&mut self, device: &'a Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            Gain {
                tag: TypeTag::Gain,
                segment: self.segment as u8,
                flag: if self.transition {
                    GainControlFlags::UPDATE
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
                crate::firmware::driver::write_to_tx(dst, self.calculator.calc(tr));
            });

        self.is_done = true;
        Ok(size_of::<Gain>() + device.len() * size_of::<Drive>())
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<'a, G: GainCalculatorGenerator<'a>> OperationGenerator<'a> for GainOperationGenerator<'a, G> {
    type O1 = GainOp<G::Calculator>;
    type O2 = NullOp;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        let c = self.generator.generate(device);
        Some((
            Self::O1::new(
                self.segment,
                self.transition_params != autd3_core::firmware::transition_mode::Later.params(),
                c,
            ),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::{
        firmware::{Intensity, Phase},
        geometry::Transducer,
    };

    use rand::prelude::*;

    use super::*;

    struct Impl {
        data: Vec<Drive>,
    }

    impl GainCalculator<'_> for Impl {
        fn calc(&self, tr: &Transducer) -> Drive {
            self.data[tr.idx()]
        }
    }

    #[test]
    fn test() {
        let device = crate::autd3_device::tests::create_device();

        let mut tx =
            vec![0x00u8; size_of::<Gain>() + device.num_transducers() * size_of::<Drive>()];

        let mut rng = rand::rng();
        let data: Vec<_> = (0..device.num_transducers())
            .map(|_| Drive {
                phase: Phase(rng.random_range(0x00..=0xFF)),
                intensity: Intensity(rng.random_range(0..=0xFF)),
            })
            .collect();

        let mut op = GainOp::new(Segment::S0, true, {
            let data = data.clone();
            Impl { data }
        });

        assert_eq!(
            op.required_size(&device),
            size_of::<Gain>() + device.num_transducers() * size_of::<Drive>()
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
                assert_eq!(d[0], &g.phase.0);
                assert_eq!(d[1], &g.intensity.0);
            });
    }
}
