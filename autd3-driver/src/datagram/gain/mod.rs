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

use rayon::prelude::*;

use super::with_segment::DatagramS;

const PARALLEL_THRESHOLD: usize = 4;

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

    fn transform<FT: Fn(&Transducer) -> Drive, F: Fn(&Device) -> FT + Sync>(
        geometry: &Geometry,
        filter: GainFilter,
        f: F,
    ) -> HashMap<usize, Vec<Drive>>
    where
        Self: Sized,
    {
        #[cfg(all(feature = "force_parallel", feature = "force_serial"))]
        compile_error!("Cannot specify both force_parallel and force_serial");
        #[cfg(all(feature = "force_parallel", not(feature = "force_serial")))]
        let n = usize::MAX;
        #[cfg(all(not(feature = "force_parallel"), feature = "force_serial"))]
        let n = 0;
        #[cfg(all(not(feature = "force_parallel"), not(feature = "force_serial")))]
        let n = geometry.devices().count();

        if n > PARALLEL_THRESHOLD {
            match filter {
                GainFilter::All => geometry
                    .devices()
                    .par_bridge()
                    .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                    .collect(),
                GainFilter::Filter(filter) => geometry
                    .devices()
                    .par_bridge()
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
        } else {
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
    ) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(segment, transition_mode.is_some(), self),
            Self::O2::default(),
        )
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{defined::FREQ_40K, derive::*, geometry::tests::create_geometry};

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
