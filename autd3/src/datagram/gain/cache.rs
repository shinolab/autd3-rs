pub use autd3_driver::derive::Gain;
pub use autd3_driver::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    geometry::Geometry,
};

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};

use derive_more::{Debug, Deref};

#[derive(Gain, Clone, Deref, Debug)]
pub struct Cache<G: Gain> {
    #[deref]
    gain: G,
    #[debug("{}", !self.cache.borrow().is_empty())]
    cache: Rc<RefCell<HashMap<usize, Arc<Vec<Drive>>>>>,
}

pub trait IntoCache<G: Gain> {
    fn with_cache(self) -> Cache<G>;
}

impl<G: Gain> IntoCache<G> for G {
    fn with_cache(self) -> Cache<G> {
        Cache::new(self)
    }
}

impl<G: Gain> Cache<G> {
    fn new(gain: G) -> Self {
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
            let mut f = self.gain.calc(geometry)?;
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
    fn calc(&self, geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
        self.init(geometry)?;
        let cache = self.cache.borrow();
        Ok(Box::new(move |dev| {
            let drives = cache[&dev.idx()].clone();
            Box::new(move |tr| drives[tr.idx()])
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{gain::Uniform, tests::create_geometry};

    use super::*;

    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let mut rng = rand::thread_rng();
        let d = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let gain = Uniform::new(d);
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

    #[derive(Gain, Clone, self::Debug)]
    pub struct CacheTestGain {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    impl Gain for CacheTestGain {
        fn calc(&self, _: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Self::transform(|_| |_| Drive::null()))
        }
    }

    #[test]
    fn test_calc_once() {
        let geometry = create_geometry(1);

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
    fn test_clone() {
        let geometry = create_geometry(1);

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
