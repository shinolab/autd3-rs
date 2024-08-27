use std::collections::HashMap;

use crate::firmware::operation::GainOp;
use crate::firmware::operation::NullOp;
use crate::firmware::operation::OperationGenerator;
use crate::{
    derive::{Geometry, Segment},
    error::AUTDInternalError,
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};

use bit_vec::BitVec;

use super::Datagram;
use super::DatagramS;

pub type GainCalcFn<'a> =
    Box<dyn FnMut(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send> + 'a>;

pub trait Gain: std::fmt::Debug {
    fn calc<'a>(&'a self, geometry: &Geometry) -> Result<GainCalcFn<'a>, AUTDInternalError>;
    fn calc_with_filter<'a>(
        &'a self,
        geometry: &Geometry,
        _filter: HashMap<usize, BitVec<u32>>,
    ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
        self.calc(geometry)
    }
    fn transform<
        'a,
        D: Into<Drive>,
        FT: Fn(&Transducer) -> D + Sync + Send + 'static,
        F: Fn(&Device) -> FT + 'a,
    >(
        f: F,
    ) -> GainCalcFn<'a>
    where
        Self: Sized,
    {
        Box::new(move |dev| {
            let f = f(dev);
            Box::new(move |tr| f(tr).into())
        })
    }
}

// GRCOV_EXCL_START
#[cfg(not(feature = "lightweight"))]
pub type BoxedGain<'a> = Box<dyn Gain + 'a>;
#[cfg(feature = "lightweight")]
pub type BoxedGain<'a> = Box<dyn Gain + Send + Sync + 'a>;

impl<'a> Gain for BoxedGain<'a> {
    fn calc(&self, geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
        self.as_ref().calc(geometry)
    }
}

impl<'a> Datagram for BoxedGain<'a> {
    type G = GainOperationGenerator<BoxedGain<'a>>;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Self::G::new(self, geometry, Segment::S0, true)
    }
}

impl<'a> DatagramS for BoxedGain<'a> {
    type G = GainOperationGenerator<BoxedGain<'a>>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition: bool,
    ) -> Result<Self::G, AUTDInternalError> {
        Self::G::new(self, geometry, segment, transition)
    }
}
// GRCOV_EXCL_STOP

pub struct GainOperationGenerator<G: Gain> {
    pub gain: std::pin::Pin<Box<G>>,
    pub g: *mut GainCalcFn<'static>,
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
            g: std::ptr::null_mut(),
            segment,
            transition,
        };
        r.g = Box::into_raw(Box::new(r.gain.calc(geometry)?)) as *mut _;
        Ok(r)
    }
}

impl<G: Gain> Drop for GainOperationGenerator<G> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.g);
        }
    }
}

impl<'a, G: Gain + 'a> OperationGenerator for GainOperationGenerator<G> {
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

    use crate::{
        derive::*,
        firmware::fpga::{EmitIntensity, Phase},
        geometry::tests::create_geometry,
    };

    #[derive(Gain, Clone, Debug)]
    pub struct TestGain {
        pub data: HashMap<usize, Vec<Drive>>,
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
            }
        }
    }

    impl Gain for TestGain {
        fn calc(&self, _geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
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
    #[cfg_attr(miri, ignore)]
    fn test_transform(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] enabled: Vec<bool>,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        let mut geometry = create_geometry(n, NUM_TRANSDUCERS);
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
        let mut f = g.calc(&geometry)?;
        assert_eq!(
            expect,
            geometry
                .devices()
                .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                .collect()
        );

        Ok(())
    }
}
