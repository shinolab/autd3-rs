pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    geometry::Geometry,
};
pub use autd3_derive::Gain;

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};

use super::GainCalcResult;

use derive_more::Deref;

#[derive(Gain, Clone, Debug, Deref)]
#[no_gain_cache]
#[no_gain_transform]
pub struct Cache<G: Gain> {
    #[deref]
    gain: G,
    cache: Rc<RefCell<HashMap<usize, Arc<Vec<Drive>>>>>,
}

pub trait IntoCache<G: Gain> {
    fn with_cache(self) -> Cache<G>;
}

impl<G: Gain> Cache<G> {
    #[doc(hidden)]
    pub fn new(gain: G) -> Self {
        Self {
            gain,
            cache: Default::default(),
        }
    }

    pub fn init(&self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.cache.borrow().len() != geometry.devices().count()
            || geometry
                .devices()
                .any(|dev| !self.cache.borrow().contains_key(&dev.idx()))
        {
            let f = self.gain.calc(geometry)?;
            geometry
                .devices()
                .filter(|dev| !self.cache.borrow().contains_key(&dev.idx()))
                .for_each(|dev| {
                    self.cache
                        .borrow_mut()
                        .insert(dev.idx(), Arc::new(dev.iter().map(f(dev)).collect()));
                });
        }
        Ok(())
    }

    pub fn drives(&self) -> Ref<'_, HashMap<usize, Arc<Vec<Drive>>>> {
        self.cache.borrow()
    }
}

impl<G: Gain> Gain for Cache<G> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.init(geometry)?;
        let cache = self.cache.borrow();
        Ok(Box::new(move |dev| {
            let drives = cache[&dev.idx()].clone();
            Box::new(move |tr| drives[tr.idx()])
        }))
    }

    #[tracing::instrument(level = "debug", skip(self, geometry), fields(cached = self.cache.borrow().len() == geometry.devices().count() && geometry.devices().all(|dev| self.cache.borrow().contains_key(&dev.idx()))))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        <G as Gain>::trace(&self.gain, geometry);
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use super::{super::tests::TestGain, *};

    use rand::Rng;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::{derive::*, geometry::tests::create_geometry};

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249);

        let mut rng = rand::thread_rng();
        let d: Drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let gain = TestGain::new(|_| |_| d, &geometry);
        let cache = gain.clone().with_cache();

        assert!(cache.drives().is_empty());
        geometry.devices().try_for_each(|dev| {
            dev.iter().try_for_each(|tr| {
                assert_eq!(
                    gain.calc(&geometry)?(dev)(tr),
                    cache.calc(&geometry)?(dev)(tr)
                );
                Result::<(), AUTDInternalError>::Ok(())
            })
        })?;
        Ok(())
    }

    #[derive(Gain, Clone, Debug)]
    pub struct CacheTestGain {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    impl Gain for CacheTestGain {
        fn calc(&self, _: &Geometry) -> GainCalcResult {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Self::transform(|_| |_| Drive::null()))
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_calc_once() {
        let geometry = create_geometry(1, 249);

        let calc_cnt = Arc::new(AtomicUsize::new(0));
        let gain = CacheTestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_clone() {
        let geometry = create_geometry(1, 249);

        let calc_cnt = Arc::new(AtomicUsize::new(0));
        let gain = CacheTestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        let g2 = gain.clone();
        assert_eq!(0, gain.calc_cnt.load(Ordering::Relaxed));
        assert_eq!(0, g2.calc_cnt.load(Ordering::Relaxed));
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let _ = g2.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = g2.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }
}
