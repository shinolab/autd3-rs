mod cache;
mod group;
mod transform;

use std::collections::HashMap;

pub use cache::Cache as GainCache;
pub use cache::IntoCache as IntoGainCache;
pub use group::Group;
pub use transform::IntoTransform as IntoGainTransform;
pub use transform::Transform as GainTransform;

use crate::firmware::operation::OperationGenerator;
use crate::{
    derive::{GainOp, Geometry, NullOp, Segment},
    error::AUTDInternalError,
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};

use bitvec::prelude::*;

use super::Datagram;
use super::DatagramS;

pub type GainCalcResult<'a> = Result<
    Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send> + 'a>,
    AUTDInternalError,
>;

pub trait Gain {
    fn calc<'a>(&'a self, geometry: &Geometry) -> GainCalcResult<'a>;
    fn calc_with_filter<'a>(
        &'a self,
        geometry: &Geometry,
        _filter: HashMap<usize, BitVec<usize, Lsb0>>,
    ) -> GainCalcResult<'a> {
        self.calc(geometry)
    }
    #[allow(clippy::type_complexity)]
    fn transform<
        'a,
        FT: Fn(&Transducer) -> Drive + Sync + Send + 'static,
        F: Fn(&Device) -> FT + 'a,
    >(
        f: F,
    ) -> Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send> + 'a>
    where
        Self: Sized,
    {
        Box::new(move |dev| {
            let f = f(dev);
            Box::new(move |tr| f(tr))
        })
    }

    #[tracing::instrument(skip(self, _geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

// GRCOV_EXCL_START
impl<'a> Gain for Box<dyn Gain + 'a> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.as_ref().calc(geometry)
    }

    #[tracing::instrument(skip(self, geometry))]
    fn trace(&self, geometry: &Geometry) {
        self.as_ref().trace(geometry);
    }
}

impl<'a> Datagram for Box<dyn Gain + 'a> {
    type O1 = GainOp;
    type O2 = NullOp;
    type G = GainOperationGenerator<Box<dyn Gain + 'a>>;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Self::G::new(self, geometry, Segment::S0, true)
    }

    #[tracing::instrument(skip(self, geometry))]
    fn trace(&self, geometry: &Geometry) {
        self.as_ref().trace(geometry);
        if tracing::enabled!(tracing::Level::DEBUG) {
            if let Ok(f) = <Self as Gain>::calc(self, geometry) {
                geometry.devices().for_each(|dev| {
                    tracing::debug!("Device[{}]", dev.idx());
                    let f = f(dev);
                    if tracing::enabled!(tracing::Level::TRACE) {
                        dev.iter().for_each(|tr| {
                            tracing::debug!("  Transducer[{}]: {}", tr.idx(), f(tr));
                        });
                    } else {
                        tracing::debug!("  Transducer[{}]: {}", 0, f(&dev[0]));
                        tracing::debug!("  ︙");
                        tracing::debug!(
                            "  Transducer[{}]: {}",
                            dev.num_transducers() - 1,
                            f(&dev[dev.num_transducers() - 1])
                        );
                    }
                });
            } else {
                tracing::error!("Failed to calculate gain");
            }
        }
    }
}

impl<'a> DatagramS for Box<dyn Gain + 'a> {
    type O1 = GainOp;
    type O2 = NullOp;
    type G = GainOperationGenerator<Box<dyn Gain + 'a>>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition: bool,
    ) -> Result<Self::G, AUTDInternalError> {
        Self::G::new(self, geometry, segment, transition)
    }

    #[tracing::instrument(skip(self, geometry))]
    fn trace(&self, geometry: &Geometry) {
        self.as_ref().trace(geometry);
        if tracing::enabled!(tracing::Level::DEBUG) {
            if let Ok(f) = <Self as Gain>::calc(self, geometry) {
                geometry.devices().for_each(|dev| {
                    tracing::debug!("Device[{}]", dev.idx());
                    let f = f(dev);
                    if tracing::enabled!(tracing::Level::TRACE) {
                        dev.iter().for_each(|tr| {
                            tracing::debug!("  Transducer[{}]: {}", tr.idx(), f(tr));
                        });
                    } else {
                        tracing::debug!("  Transducer[{}]: {}", 0, f(&dev[0]));
                        tracing::debug!("  ︙");
                        tracing::debug!(
                            "  Transducer[{}]: {}",
                            dev.num_transducers() - 1,
                            f(&dev[dev.num_transducers() - 1])
                        );
                    }
                });
            } else {
                tracing::error!("Failed to calculate gain");
            }
        }
    }
}

impl<'a> Gain for Box<dyn Gain + Send + Sync + 'a> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.as_ref().calc(geometry)
    }

    #[tracing::instrument(skip(self, geometry))]
    fn trace(&self, geometry: &Geometry) {
        self.as_ref().trace(geometry);
    }
}

#[cfg(feature = "capi")]
mod capi {
    use crate::derive::*;

    #[derive(Gain)]
    struct NullGain {}

    impl<'a> Gain for NullGain {
        fn calc(&self, _: &Geometry) -> GainCalcResult {
            Ok(Box::new(move |_| Box::new(move |_| Drive::null())))
        }
    }

    impl<'a> Default for Box<dyn Gain + 'a> {
        fn default() -> Self {
            Box::new(NullGain {})
        }
    }
}
// GRCOV_EXCL_STOP

pub struct GainOperationGenerator<G: Gain> {
    pub gain: std::pin::Pin<Box<G>>,
    #[allow(clippy::type_complexity)]
    pub g: *const Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>,
    pub segment: Segment,
    pub transition: bool,
}

impl<G: Gain> GainOperationGenerator<G> {
    pub fn new(
        gain: G,
        geometry: &Geometry,
        segment: Segment,
        transition: bool,
    ) -> Result<Self, AUTDInternalError> {
        let mut r = Self {
            gain: Box::pin(gain),
            g: std::ptr::null(),
            segment,
            transition,
        };
        let g = Box::new(r.gain.calc(geometry)?)
            as Box<Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>>;
        r.g = Box::into_raw(g) as *const _;
        Ok(r)
    }
}

impl<G: Gain> Drop for GainOperationGenerator<G> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(
                self.g
                    as *mut Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>,
            );
        }
    }
}

impl<G: Gain> OperationGenerator for GainOperationGenerator<G> {
    type O1 = GainOp;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        let d = unsafe { (*self.g)(device) };
        (
            GainOp::new(self.segment, self.transition, d),
            NullOp::default(),
        )
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
            geometry
                .devices()
                .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                .collect()
        );
    }
}
