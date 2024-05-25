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

use super::GainCalcFn;

#[derive(Gain, Debug)]
#[no_gain_cache]
#[no_gain_transform]
pub struct Cache<G: Gain + 'static> {
    gain: Arc<G>,
    cache: Arc<RwLock<HashMap<usize, Vec<Drive>>>>,
}

impl<G: Gain + 'static> std::ops::Deref for Cache<G> {
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.gain
    }
}

pub trait IntoCache<G: Gain + 'static> {
    fn with_cache(self) -> Cache<G>;
}

impl<G: Gain + Clone + 'static> Clone for Cache<G> {
    fn clone(&self) -> Self {
        Self {
            gain: self.gain.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<G: Gain + 'static> Cache<G> {
    #[doc(hidden)]
    pub fn new(gain: G) -> Self {
        Self {
            gain: Arc::new(gain),
            cache: Arc::new(Default::default()),
        }
    }

    pub fn init(&self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.cache.read().unwrap().len() != geometry.devices().count()
            || geometry
                .devices()
                .any(|dev| !self.cache.read().unwrap().contains_key(&dev.idx()))
        {
            let f = self.gain.calc(geometry, GainFilter::All)?;
            *self.cache.write().unwrap() = geometry
                .devices()
                .map(|dev| {
                    (dev.idx(), {
                        let f = f(dev);
                        dev.iter().map(|tr| f(tr)).collect()
                    })
                })
                .collect();
        }
        Ok(())
    }

    pub fn drives(&self) -> RwLockReadGuard<HashMap<usize, Vec<Drive>>> {
        self.cache.read().unwrap()
    }
}

impl<G: Gain + 'static> Gain for Cache<G> {
    fn calc<'a>(
        &'a self,
        geometry: &'a Geometry,
        _filter: GainFilter<'a>,
    ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
        self.init(geometry)?;
        Ok(Box::new(|dev| {
            let cache = self.drives()[&dev.idx()].clone();
            Box::new(move |tr| cache[tr.idx()])
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
        let gain = TestGain {
            f: move |_| move |_| d,
        };
        let cache = gain.with_cache();

        assert!(cache.drives().is_empty());
        geometry.devices().try_for_each(|dev| {
            dev.iter().try_for_each(|tr| {
                assert_eq!(
                    gain.calc(&geometry, GainFilter::All)?(dev)(tr),
                    cache.calc(&geometry, GainFilter::All)?(dev)(tr)
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
        fn calc<'a>(
            &'a self,
            _: &'a Geometry,
            filter: GainFilter<'a>,
        ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Self::transform(
                filter,
                Box::new(|_| Box::new(|_| Drive::null())),
            ))
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
        let _ = gain.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All);
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

        let _ = g2.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = g2.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }
}
