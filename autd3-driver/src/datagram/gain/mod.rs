mod cache;
mod group;
mod transform;

use std::collections::HashMap;

pub use cache::Cache as GainCache;
pub use cache::IntoCache as IntoGainCache;
pub use group::Group;
pub use transform::IntoTransform as IntoGainTransform;
pub use transform::Transform as GainTransform;

use super::OperationGenerator;
use crate::{
    derive::{GainOp, Geometry, NullOp, Segment},
    error::AUTDInternalError,
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};

use bitvec::prelude::*;

pub type GainCalcResult =
    Result<Box<dyn Fn(&Device) -> Vec<Drive> + Send + Sync>, AUTDInternalError>;

pub trait Gain {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult;
    fn calc_with_filter(
        &self,
        geometry: &Geometry,
        _filter: HashMap<usize, BitVec<usize, Lsb0>>,
    ) -> GainCalcResult {
        self.calc(geometry)
    }
    #[allow(clippy::type_complexity)]
    fn transform<FT: Fn(&Transducer) -> Drive, F: Fn(&Device) -> FT + Send + Sync + 'static>(
        f: F,
    ) -> Box<dyn Fn(&Device) -> Vec<Drive> + Send + Sync>
    where
        Self: Sized,
    {
        Box::new(move |dev| dev.iter().map(f(dev)).collect())
    }
}

impl Gain for Box<dyn Gain> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.as_ref().calc(geometry)
    }
}

pub struct GainOperationGenerator<'a> {
    #[allow(clippy::type_complexity)]
    pub g: Box<dyn Fn(&Device) -> Vec<Drive> + Send + Sync + 'a>,
    pub segment: Segment,
    pub transition: bool,
}

impl<'a> OperationGenerator<'a> for GainOperationGenerator<'a> {
    type O1 = GainOp;
    type O2 = NullOp;

    fn generate(&'a self, device: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let d = (self.g)(device);
        Ok((
            GainOp::new(self.segment, self.transition, d),
            NullOp::default(),
        ))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::{defined::FREQ_40K, derive::*, geometry::tests::create_geometry};

    #[derive(Gain, Clone)]
    pub struct TestGain {
        pub data: HashMap<usize, Vec<Drive>>,
        pub err: Option<AUTDInternalError>,
    }

    impl TestGain {
        pub fn new<FT: Fn(&Transducer) -> Drive, F: Fn(&Device) -> FT>(
            f: F,
            geometry: &Geometry,
        ) -> Self {
            Self {
                data: geometry
                    .devices()
                    .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                    .collect(),
                err: None,
            }
        }

        pub fn null(geometry: &Geometry) -> Self {
            Self {
                data: geometry
                    .devices()
                    .map(|dev| (dev.idx(), vec![Drive::null(); dev.num_transducers()]))
                    .collect(),
                err: None,
            }
        }

        pub fn err() -> Self {
            Self {
                data: Default::default(),
                err: Some(AUTDInternalError::GainError("test".to_owned())),
            }
        }
    }

    impl Gain for TestGain {
        fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
            if let Some(ref err) = self.err {
                return Err(err.clone());
            }
            let d = self.data.clone();
            Ok(Self::transform(move |dev| {
                let d = d[&dev.idx()].clone();
                move |tr| d[tr.idx()]
            }))
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
    fn test_transform(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] enabled: Vec<bool>,
        #[case] n: usize,
    ) {
        let mut geometry = create_geometry(n, NUM_TRANSDUCERS, FREQ_40K);
        geometry
            .iter_mut()
            .zip(enabled.iter())
            .for_each(|(dev, &e)| dev.enable = e);
        let g = TestGain::new(
            |dev| {
                let dev_idx = dev.idx();
                move |_| {
                    Drive::new(
                        Phase::new(dev_idx as u8 + 1),
                        EmitIntensity::new(dev_idx as u8 + 1),
                    )
                }
            },
            &geometry,
        );
        let f = g.calc(&geometry).unwrap();
        assert_eq!(
            expect,
            geometry.devices().map(|dev| (dev.idx(), f(dev))).collect()
        );
    }
}
