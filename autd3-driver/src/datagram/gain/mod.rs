mod cache;
mod group;
mod transform;

use std::collections::HashMap;

pub use cache::Cache as GainCache;
pub use cache::IntoCache as IntoGainCache;
pub use group::Group;
pub use transform::IntoTransform as IntoGainTransform;
pub use transform::Transform as GainTransform;

use crate::{
    derive::Geometry,
    error::AUTDInternalError,
    firmware::{
        fpga::{Drive, Segment},
        operation::{GainOp, NullOp},
    },
    geometry::{Device, Transducer},
};

use bitvec::prelude::*;

use super::with_segment::DatagramS;

pub enum GainFilter<'a> {
    All,
    Filter(&'a HashMap<usize, BitVec<usize, Lsb0>>),
}

pub type GainCalcFn<'a> =
    Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + 'a> + Send + Sync + 'a>;

pub trait Gain {
    fn calc<'a>(
        &'a self,
        geometry: &'a Geometry,
        filter: GainFilter<'a>,
    ) -> Result<GainCalcFn<'a>, AUTDInternalError>;
    fn transform<'a>(filter: GainFilter<'a>, f: GainCalcFn<'a>) -> GainCalcFn<'a>
    where
        Self: Sized,
    {
        match filter {
            GainFilter::All => f,
            GainFilter::Filter(filter) => Box::new(move |dev| {
                let filter = filter.get(&dev.idx());
                let ft = f(dev);
                Box::new(move |tr| match filter {
                    Some(f) if f[tr.idx()] => ft(tr),
                    _ => Drive::null(),
                })
            }),
        }
    }
}

// GRCOV_EXCL_START
impl Gain for Box<dyn Gain> {
    fn calc<'a>(
        &'a self,
        geometry: &'a Geometry,
        filter: GainFilter<'a>,
    ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
        self.as_ref().calc(geometry, filter)
    }
}

impl<'a> DatagramS<'a> for Box<dyn Gain> {
    type O1 = GainOp<'a>;
    type O2 = NullOp;

    fn operation_with_segment(
        &'a self,
        geometry: &'a Geometry,
        segment: Segment,
        transition: bool,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        let f = self.calc(geometry, GainFilter::All)?;
        Ok(move |dev| (GainOp::new(segment, transition, f(dev)), NullOp::default()))
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{defined::FREQ_40K, derive::*, geometry::tests::create_geometry};

    #[derive(Gain, Clone, Copy, PartialEq, Debug)]
    pub struct TestGain<FT, F>
    where
        FT: Fn(&Transducer) -> Drive + 'static,
        F: Fn(&Device) -> FT + Send + Sync + 'static,
    {
        pub f: F,
    }

    impl
        TestGain<
            Box<dyn Fn(&Transducer) -> Drive>,
            Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive> + Send + Sync>,
        >
    {
        pub fn null() -> Self {
            Self {
                f: Box::new(|_| Box::new(|_| Drive::null())),
            }
        }
    }

    impl<FT: Fn(&Transducer) -> Drive + 'static, F: Fn(&Device) -> FT + Send + Sync + 'static> Gain
        for TestGain<FT, F>
    {
        fn calc<'a>(
            &'a self,
            _geometry: &'a Geometry,
            filter: GainFilter<'a>,
        ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
            Ok(Self::transform(
                filter,
                Box::new(move |dev| {
                    let f = (self.f)(dev);
                    Box::new(move |tr| f(tr))
                }),
            ))
        }
    }

    #[derive(Gain, Copy, Clone)]
    pub struct ErrGain {}

    impl Gain for ErrGain {
        fn calc(
            &self,
            _geometry: &Geometry,
            _filter: GainFilter,
        ) -> Result<GainCalcFn, AUTDInternalError> {
            Err(AUTDInternalError::GainError("test".to_owned()))
        }
    }

    const NUM_TRANSDUCERS: usize = 2;

    #[rstest::rstest]
    #[test]
    #[case::serial(
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
            (1, vec![Drive::new(Phase::new(0x02), EmitIntensity::new(0x02)); NUM_TRANSDUCERS])
        ].into_iter().collect(),
        vec![true; 2],
        2)]
    #[case::parallel(
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
            (1, vec![Drive::new(Phase::new(0x02), EmitIntensity::new(0x02)); NUM_TRANSDUCERS]),
            (2, vec![Drive::new(Phase::new(0x03), EmitIntensity::new(0x03)); NUM_TRANSDUCERS]),
            (3, vec![Drive::new(Phase::new(0x04), EmitIntensity::new(0x04)); NUM_TRANSDUCERS]),
            (4, vec![Drive::new(Phase::new(0x05), EmitIntensity::new(0x05)); NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        vec![true; 5],
        5)]
    #[case::enabled(
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        vec![true, false],
        2)]
    fn test_transform_all(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] enabled: Vec<bool>,
        #[case] n: usize,
    ) {
        let mut geometry = create_geometry(n, NUM_TRANSDUCERS, FREQ_40K);
        geometry
            .iter_mut()
            .zip(enabled.iter())
            .for_each(|(dev, &e)| dev.enable = e);
        let g = TestGain {
            f: |dev| {
                let dev_idx = dev.idx();
                move |_| {
                    Drive::new(
                        Phase::new(dev_idx as u8 + 1),
                        EmitIntensity::new(dev_idx as u8 + 1),
                    )
                }
            },
        };
        assert_eq!(
            expect,
            geometry
                .devices()
                .map(|dev| (dev.idx(), {
                    let f = g.calc(&geometry, GainFilter::All).unwrap()(dev);
                    dev.iter().map(|tr| f(tr)).collect()
                }))
                .collect()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
    [
        (0, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x01), EmitIntensity::new(0x01))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect()),
        (1, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x02), EmitIntensity::new(0x02))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect())
    ].into_iter().collect(),
    vec![true; 2],
    [
        (0, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
        (1, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
    ].iter().cloned().collect(),
    2)]
    #[case::parallel(
    [
        (0, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x01), EmitIntensity::new(0x01))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect()),
        (1, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x02), EmitIntensity::new(0x02))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect()),
        (2, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x03), EmitIntensity::new(0x03))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect()),
        (3, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x04), EmitIntensity::new(0x04))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect()),
        (4, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x05), EmitIntensity::new(0x05))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect()),
    ].into_iter().collect(),
    vec![true; 5],
    [
        (0, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
        (1, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
        (2, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
        (3, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
        (4, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
    ].iter().cloned().collect(),
    5)]
    #[case::enabled(
    [
        (1, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x02), EmitIntensity::new(0x02))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect())
    ].into_iter().collect(),
    vec![false, true],
    [
        (1, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
    ].iter().cloned().collect(),
    2)]
    fn test_transform_filtered(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] enabled: Vec<bool>,
        #[case] filter: HashMap<usize, BitVec<usize, Lsb0>>,
        #[case] n: usize,
    ) {
        use crate::defined::FREQ_40K;

        let mut geometry = create_geometry(n, NUM_TRANSDUCERS, FREQ_40K);
        geometry
            .iter_mut()
            .zip(enabled.iter())
            .for_each(|(dev, &e)| dev.enable = e);

        let g = TestGain {
            f: |dev| {
                let dev_idx = dev.idx();
                move |_| {
                    Drive::new(
                        Phase::new(dev_idx as u8 + 1),
                        EmitIntensity::new(dev_idx as u8 + 1),
                    )
                }
            },
        };
        assert_eq!(
            expect,
            geometry
                .devices()
                .map(|dev| (dev.idx(), {
                    let f = g.calc(&geometry, GainFilter::Filter(&filter)).unwrap()(dev);
                    dev.iter().map(|tr| f(tr)).collect()
                }))
                .collect()
        );
    }
}
