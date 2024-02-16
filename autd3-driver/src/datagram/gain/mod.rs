mod cache;
mod group;
mod transform;

pub use cache::Cache as GainCache;
pub use cache::IntoCache as IntoGainCache;
pub use group::Group;
pub use transform::IntoTransform as IntoGainTransform;
pub use transform::Transform as GainTransform;

use std::collections::HashMap;
use std::time::Duration;

use crate::{
    common::{Drive, Segment},
    error::AUTDInternalError,
    geometry::{Device, Geometry, Transducer},
    operation::{GainOp, NullOp},
};

use bitvec::prelude::*;

use super::with_segment::DatagramS;
use super::Datagram;

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
    fn transform(
        geometry: &Geometry,
        filter: GainFilter,
        f: impl Fn(&Device, &Transducer) -> Drive,
    ) -> HashMap<usize, Vec<Drive>>
    where
        Self: Sized,
    {
        match filter {
            GainFilter::All => geometry
                .devices()
                .map(|dev| (dev.idx(), dev.iter().map(|tr| f(dev, tr)).collect()))
                .collect(),
            GainFilter::Filter(filter) => geometry
                .devices()
                .filter_map(|dev| {
                    filter.get(&dev.idx()).map(|filter| {
                        (
                            dev.idx(),
                            dev.iter()
                                .map(|tr| {
                                    if filter[tr.idx()] {
                                        f(dev, tr)
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

impl<'a> Gain for Box<dyn Gain + 'a> {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        self.as_ref().calc(geometry, filter)
    }
}

impl<'a> Gain for Box<dyn Gain + Send + 'a> {
    #[cfg_attr(coverage_nightly, coverage(off))]
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

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn operation_with_segment(
        self,
        segment: Segment,
        update_segment: bool,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((
            Self::O1::new(segment, update_segment, self),
            Self::O2::default(),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChangeGainSegment {
    segment: Segment,
}

impl ChangeGainSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }
}

impl Datagram for ChangeGainSegment {
    type O1 = crate::operation::GainChangeSegmentOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.segment), Self::O2::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{derive::*, geometry::tests::create_geometry};

    #[derive(Gain, Clone, Copy, PartialEq, Debug)]
    pub struct TestGain {
        pub d: Drive,
    }

    impl Gain for TestGain {
        fn calc(
            &self,
            geometry: &Geometry,
            filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            Ok(Self::transform(geometry, filter, |_, _| self.d))
        }
    }

    #[test]
    fn test_gain_transform_all() -> anyhow::Result<()> {
        let geometry = create_geometry(2, 249);

        let d = Drive::random();
        assert_eq!(
            geometry
                .devices()
                .map(|dev| (dev.idx(), vec![d; dev.num_transducers()]))
                .collect::<HashMap<_, _>>(),
            TestGain { d }.calc(&geometry, GainFilter::All)?
        );

        Ok(())
    }

    #[test]
    fn test_gain_transform_all_enabled() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 249);
        geometry[0].enable = false;

        let d = Drive::random();
        assert_eq!(
            geometry
                .devices()
                .map(|dev| (dev.idx(), vec![d; dev.num_transducers()]))
                .collect::<HashMap<_, _>>(),
            TestGain { d }.calc(&geometry, GainFilter::All)?
        );

        Ok(())
    }

    #[test]
    fn test_gain_transform_filtered() -> anyhow::Result<()> {
        let geometry = create_geometry(3, 249);

        let filter = geometry
            .iter()
            .take(2)
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        let d = Drive::random();
        assert_eq!(
            (0..2)
                .map(|idx| (
                    geometry[idx].idx(),
                    (0..100)
                        .map(|_| d)
                        .chain((0..).map(|_| Drive::null()))
                        .take(geometry[idx].num_transducers())
                        .collect::<Vec<_>>()
                ))
                .collect::<HashMap<_, _>>(),
            TestGain { d }.calc(&geometry, GainFilter::Filter(&filter))?
        );

        Ok(())
    }

    #[test]
    fn test_gain_transform_filtered_enabled() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 249);
        geometry[0].enable = false;

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        let d = Drive::random();

        assert_eq!(
            geometry
                .devices()
                .map(|dev| (
                    dev.idx(),
                    (0..100)
                        .map(|_| d)
                        .chain((0..).map(|_| Drive::null()))
                        .take(dev.num_transducers())
                        .collect::<Vec<_>>()
                ))
                .collect::<HashMap<_, _>>(),
            TestGain { d }.calc(&geometry, GainFilter::Filter(&filter))?
        );

        Ok(())
    }
}
