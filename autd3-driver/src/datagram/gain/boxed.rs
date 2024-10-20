use std::{collections::HashMap, time::Duration};

use super::{Gain, GainContextGenerator, GainOperationGenerator};
pub use crate::firmware::operation::GainContext;
use crate::{
    derive::{DatagramS, Geometry, Segment, TransitionMode},
    error::AUTDInternalError,
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};

use bit_vec::BitVec;

impl GainContext for Box<dyn GainContext> {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.as_ref().calc(tr)
    }
}

#[allow(clippy::type_complexity)]
pub struct BoxedGainContextGenerator {
    f: Box<dyn FnMut(&Device) -> Box<dyn GainContext>>,
}

impl GainContextGenerator for BoxedGainContextGenerator {
    type Context = Box<dyn GainContext>;

    fn generate(&mut self, device: &Device) -> Self::Context {
        (self.f)(device)
    }
}

#[cfg(not(feature = "lightweight"))]
type BoxedGen = Box<
    dyn FnOnce(
        &Geometry,
        Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<BoxedGainContextGenerator, AUTDInternalError>,
>;
#[cfg(not(feature = "lightweight"))]
type BoxedFmt = Box<dyn Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result>;
#[cfg(feature = "lightweight")]
type BoxedGen = Box<
    dyn FnOnce(
            &Geometry,
            Option<HashMap<usize, BitVec<u32>>>,
        ) -> Result<BoxedGainContextGenerator, AUTDInternalError>
        + Send
        + Sync,
>;
#[cfg(feature = "lightweight")]
type BoxedFmt = Box<dyn Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result + Send + Sync>;

pub struct BoxedGain {
    dbg: BoxedFmt,
    g: BoxedGen,
}

impl std::fmt::Debug for BoxedGain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.dbg)(f)
    }
}

impl Gain for BoxedGain {
    type G = BoxedGainContextGenerator;

    fn init_with_filter(
        self,
        geometry: &Geometry,
        filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        (self.g)(geometry, filter)
    }
}

// GRCOV_EXCL_START
impl DatagramS for BoxedGain {
    type G = GainOperationGenerator<BoxedGainContextGenerator>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        Self::G::new(self, geometry, segment, transition_mode)
    }

    fn timeout(&self) -> Option<Duration> {
        None
    }

    fn parallel_threshold(&self) -> Option<usize> {
        None
    }
}
// GRCOV_EXCL_STOP

pub trait IntoBoxedGain {
    fn into_boxed(self) -> BoxedGain;
}

#[cfg(not(feature = "lightweight"))]
impl<G: Gain> IntoBoxedGain for G
where
    G: 'static,
{
    fn into_boxed(self) -> BoxedGain {
        let gain = std::rc::Rc::new(std::cell::RefCell::new(Some(self)));
        BoxedGain {
            dbg: Box::new({
                let gain = gain.clone();
                move |f| gain.borrow().as_ref().unwrap().fmt(f)
            }),
            g: Box::new(
                move |geometry: &Geometry, filter: Option<HashMap<usize, BitVec<u32>>>| {
                    let mut f = gain.take().unwrap().init_with_filter(geometry, filter)?;
                    Ok(BoxedGainContextGenerator {
                        f: Box::new(move |dev| Box::new(f.generate(dev))),
                    })
                },
            ),
        }
    }
}

#[cfg(feature = "lightweight")]
impl<G: Gain> IntoBoxedGain for G
where
    G: Send + Sync + 'static,
{
    fn into_boxed(self) -> BoxedGain {
        let gain = std::sync::Arc::new(std::sync::Mutex::new(Some(self)));
        BoxedGain {
            dbg: Box::new({
                let gain = gain.clone();
                move |f| gain.lock().unwrap().as_ref().unwrap().fmt(f)
            }),
            g: Box::new(
                move |geometry: &Geometry, filter: Option<HashMap<usize, BitVec<u32>>>| {
                    let mut f = gain
                        .lock()
                        .unwrap()
                        .take()
                        .unwrap()
                        .init_with_filter(geometry, filter)?;
                    Ok(BoxedGainContextGenerator {
                        f: Box::new(move |dev| Box::new(f.generate(dev))),
                    })
                },
            ),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::datagram::gain::tests::TestGain;

    use crate::{
        derive::*,
        firmware::fpga::{EmitIntensity, Phase},
        geometry::tests::create_geometry,
    };

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
    fn boxed_gain(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] enabled: Vec<bool>,
        #[case] n: u16,
    ) -> anyhow::Result<()> {
        let mut geometry = create_geometry(n, NUM_TRANSDUCERS as _);
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
        )
        .into_boxed();
        let mut f = g.init(&geometry)?;
        assert_eq!(
            expect,
            geometry
                .devices()
                .map(|dev| {
                    let f = f.generate(dev);
                    (dev.idx(), dev.iter().map(|tr| f.calc(tr)).collect())
                })
                .collect()
        );

        Ok(())
    }

    #[test]
    fn boxed_gain_dbg() {
        let g = TestGain::null();
        assert_eq!(format!("{:?}", g), format!("{:?}", g.into_boxed()));
    }
}
