pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    firmware::operation::{GainOp, NullOp},
    geometry::Geometry,
};
pub use autd3_derive::Gain;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use super::GainCalcResult;

#[derive(Gain, Debug)]
#[no_gain_cache]
#[no_gain_transform]
pub struct Cache<G: Gain> {
    gain: G,
    cache: Arc<RwLock<HashMap<usize, Vec<Drive>>>>,
}

pub trait IntoCache<G: Gain> {
    fn with_cache(self) -> Cache<G>;
}

impl<G: Gain> std::ops::Deref for Cache<G> {
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.gain
    }
}

impl<G: Gain + Clone> Clone for Cache<G> {
    fn clone(&self) -> Self {
        Self {
            gain: self.gain.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<G: Gain> Cache<G> {
    #[doc(hidden)]
    pub fn new(gain: G) -> Self {
        Self {
            gain,
            cache: Arc::new(Default::default()),
        }
    }

    pub fn init(&self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.cache.read().unwrap().len() != geometry.devices().count()
            || geometry
                .devices()
                .any(|dev| !self.cache.read().unwrap().contains_key(&dev.idx()))
        {
            let f = self.gain.calc(geometry)?;
            *self.cache.write().unwrap() = geometry
                .devices()
                .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                .collect();
        }
        Ok(())
    }

    pub fn drives(&self) -> RwLockReadGuard<HashMap<usize, Vec<Drive>>> {
        self.cache.read().unwrap()
    }
}

impl<G: Gain> Gain for Cache<G> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.init(geometry)?;
        let drives = self.drives().clone();
        Ok(Box::new(move |dev| {
            let drives = drives[&dev.idx()].clone();
            Box::new(move |tr| drives[tr.idx()])
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::TestGain, *};

    use rand::Rng;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::{defined::FREQ_40K, derive::*, geometry::tests::create_geometry};

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249, FREQ_40K);

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

    impl PartialEq for CacheTestGain {
        fn eq(&self, other: &Self) -> bool {
            self.calc_cnt.load(Ordering::Relaxed) == other.calc_cnt.load(Ordering::Relaxed)
        }
    }

    impl Gain for CacheTestGain {
        fn calc(&self, _: &Geometry) -> GainCalcResult {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Self::transform(|_| |_| Drive::null()))
        }
    }

    #[test]
    fn test_calc_once() {
        let geometry = create_geometry(1, 249, FREQ_40K);

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
        let geometry = create_geometry(1, 249, FREQ_40K);

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
