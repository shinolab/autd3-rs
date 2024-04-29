mod cache;
mod group;
mod segment;
mod transform;

pub use cache::Cache as GainCache;
pub use cache::IntoCache as IntoGainCache;
pub use group::Group;
pub use segment::ChangeGainSegment;
pub use transform::IntoTransform as IntoGainTransform;
pub use transform::Transform as GainTransform;

use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Drive, Segment, TransitionMode},
        operation::{GainOp, NullOp},
    },
    geometry::{Device, Geometry, Transducer},
};

use bitvec::prelude::*;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use super::with_segment::DatagramS;

pub enum GainFilter<'a> {
    All,
    Filter(&'a HashMap<usize, BitVec<usize, Lsb0>>),
}

/// Gain controls amplitude and phase of each transducer.
pub trait Gain {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError>;

    #[cfg(not(feature = "parallel"))]
    fn transform<FT: Fn(&Transducer) -> Drive, F: Fn(&Device) -> FT>(
        geometry: &Geometry,
        filter: GainFilter,
        f: F,
    ) -> HashMap<usize, Vec<Drive>>
    where
        Self: Sized,
    {
        match filter {
            GainFilter::All => geometry
                .devices()
                .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                .collect(),
            GainFilter::Filter(filter) => geometry
                .devices()
                .filter_map(|dev| {
                    filter.get(&dev.idx()).map(|filter| {
                        let ft = f(dev);
                        (
                            dev.idx(),
                            dev.iter()
                                .map(|tr| {
                                    if filter[tr.idx()] {
                                        ft(tr)
                                    } else {
                                        Drive::null()
                                    }
                                })
                                .collect(),
                        )
                    })
                })
                .collect(),
        }
    }

    #[cfg(feature = "parallel")]
    fn transform<FT: Fn(&Transducer) -> Drive, F: Fn(&Device) -> FT + Sync>(
        geometry: &Geometry,
        filter: GainFilter,
        f: F,
    ) -> HashMap<usize, Vec<Drive>>
    where
        Self: Sized,
    {
        match filter {
            GainFilter::All => geometry
                .devices()
                .par_bridge()
                .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                .collect(),
            GainFilter::Filter(filter) => geometry
                .devices()
                // .par_bridge()
                .filter_map(|dev| {
                    filter.get(&dev.idx()).map(|filter| {
                        let ft = f(dev);
                        (
                            dev.idx(),
                            dev.iter()
                                .map(|tr| {
                                    if filter[tr.idx()] {
                                        ft(tr)
                                    } else {
                                        Drive::null()
                                    }
                                })
                                .collect(),
                        )
                    })
                })
                .collect(),
        }
    }
}

// GRCOV_EXCL_START
impl<'a> Gain for Box<dyn Gain + 'a> {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        self.as_ref().calc(geometry, filter)
    }
}

impl<'a> Gain for Box<dyn Gain + Send + 'a> {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        self.as_ref().calc(geometry, filter)
    }
}

impl DatagramS for Box<dyn Gain> {
    type O1 = GainOp<Self>;
    type O2 = NullOp;

    fn operation_with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((
            Self::O1::new(segment, transition_mode.is_some(), self),
            Self::O2::default(),
        ))
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{derive::*, geometry::tests::create_geometry};

    #[derive(Gain, Clone, Copy, PartialEq, Debug)]
    pub struct TestGain<
        FT: Fn(&Transducer) -> Drive + 'static,
        F: Fn(&Device) -> FT + Sync + 'static,
    > {
        pub f: F,
    }

    impl
        TestGain<
            Box<dyn Fn(&Transducer) -> Drive>,
            Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive> + Sync>,
        >
    {
        pub fn null() -> Self {
            Self {
                f: Box::new(|_| Box::new(|_| Drive::null())),
            }
        }
    }

    impl<FT: Fn(&Transducer) -> Drive + 'static, F: Fn(&Device) -> FT + Sync + 'static> Gain
        for TestGain<FT, F>
    {
        fn calc(
            &self,
            geometry: &Geometry,
            filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            Ok(Self::transform(geometry, filter, &self.f))
        }
    }

    const NUM_TRANSDUCERS: usize = 2;

    #[rstest::fixture]
    fn geometry() -> Geometry {
        create_geometry(2, NUM_TRANSDUCERS)
    }

    #[rstest::rstest]
    #[test]
    #[case(
        [true, true],
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
            (1, vec![Drive::new(Phase::new(0x02), EmitIntensity::new(0x02)); NUM_TRANSDUCERS])
        ].into_iter().collect())]
    #[case::enabled(
        [true, false],
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
        ].into_iter().collect())]
    fn test_transform_all(
        #[case] enabled: [bool; 2],
        #[case] expect: HashMap<usize, Vec<Drive>>,
        mut geometry: Geometry,
    ) {
        geometry
            .iter_mut()
            .zip(enabled.iter())
            .for_each(|(dev, &e)| dev.enable = e);
        assert_eq!(
            Ok(expect),
            TestGain {
                f: |dev| {
                    let dev_idx = dev.idx();
                    move |_| {
                        Drive::new(
                            Phase::new(dev_idx as u8 + 1),
                            EmitIntensity::new(dev_idx as u8 + 1),
                        )
                    }
                }
            }
            .calc(&geometry, GainFilter::All)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        [true, true],
        [
            (0, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x01), EmitIntensity::new(0x01))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect()),
            (1, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x02), EmitIntensity::new(0x02))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect())
        ].into_iter().collect(), [
            (0, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
            (1, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
        ].iter().cloned().collect())]
    #[case::enabled(
        [false, true],
        [
            (1, (0..NUM_TRANSDUCERS / 2).map(|_| Drive::new(Phase::new(0x02), EmitIntensity::new(0x02))).chain((0..).map(|_| Drive::null())).take(NUM_TRANSDUCERS).collect())
        ].into_iter().collect(),[
            (1, (0..NUM_TRANSDUCERS).map(|i| i < NUM_TRANSDUCERS / 2).collect()),
        ].iter().cloned().collect())]
    fn test_transform_filtered(
        #[case] enabled: [bool; 2],
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] filter: HashMap<usize, BitVec<usize, Lsb0>>,
        mut geometry: Geometry,
    ) {
        geometry
            .iter_mut()
            .zip(enabled.iter())
            .for_each(|(dev, &e)| dev.enable = e);
        assert_eq!(
            Ok(expect),
            TestGain {
                f: |dev| {
                    let dev_idx = dev.idx();
                    move |_| {
                        Drive::new(
                            Phase::new(dev_idx as u8 + 1),
                            EmitIntensity::new(dev_idx as u8 + 1),
                        )
                    }
                }
            }
            .calc(&geometry, GainFilter::Filter(&filter))
        );
    }
}
