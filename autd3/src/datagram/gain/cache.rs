pub use autd3_driver::derive::Gain;
pub use autd3_driver::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    geometry::Geometry,
};

use std::{cell::RefCell, rc::Rc};

use derive_more::Debug;

/// Cache for `Gain`s
///
/// This `Gain` is used to cache the calculated phases and intensities for each transducer.
#[derive(Gain, Debug, Builder)]
pub struct Cache<G: Gain> {
    gain: Rc<RefCell<Option<G>>>,
    #[debug("{}", !self.cache.borrow().is_empty())]
    #[get]
    /// Cached phases and intensities.
    cache: Rc<RefCell<HashMap<usize, Arc<Vec<Drive>>>>>,
}

impl<G: Gain> Clone for Cache<G> {
    fn clone(&self) -> Self {
        Self {
            gain: self.gain.clone(),
            cache: self.cache.clone(),
        }
    }
}

/// Trait to convert `Gain` to [Cache].
pub trait IntoCache<G: Gain> {
    /// Convert Gain to [Cache]
    fn with_cache(self) -> Cache<G>;
}

impl<G: Gain> IntoCache<G> for G {
    fn with_cache(self) -> Cache<G> {
        Cache {
            gain: Rc::new(RefCell::new(Some(self))),
            cache: Default::default(),
        }
    }
}

impl<G: Gain> Cache<G> {
    /// Initialize cache
    ///
    /// # Errors
    ///
    /// Returns [`AUTDInternalError::GainError`] if you initialize with some devices disabled and then reinitialize after enabling the devices.
    ///
    /// [`AUTDInternalError::GainError`]: autd3_driver::error::AUTDInternalError::GainError
    pub fn init(&self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if let Some(gain) = self.gain.take() {
            let mut f = gain.init(geometry, None)?;
            geometry
                .devices()
                .filter(|dev| !self.cache.borrow().contains_key(&dev.idx()))
                .for_each(|dev| {
                    tracing::debug!("Initializing cache for device {}", dev.idx());
                    let f = f.generate(dev);
                    self.cache.borrow_mut().insert(
                        dev.idx(),
                        Arc::new(dev.iter().map(|tr| f.calc(tr)).collect()),
                    );
                });
        }

        if self.cache.borrow().len() != geometry.devices().count()
            || geometry
                .devices()
                .any(|dev| !self.cache.borrow().contains_key(&dev.idx()))
        {
            return Err(AUTDInternalError::GainError(
                "Cache is initialized with different geometry".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the number of references to the cache
    pub fn count(&self) -> usize {
        Rc::strong_count(&self.cache)
    }
}

pub struct Context {
    g: Arc<Vec<Drive>>,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.g[tr.idx()]
    }
}

impl<G: Gain> GainContextGenerator for Cache<G> {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            g: self.cache.borrow()[&device.idx()].clone(),
        }
    }
}

impl<G: Gain> Gain for Cache<G> {
    type G = Self;

    fn init(
        self,
        geometry: &Geometry,
        _filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        Cache::init(&self, geometry)?;
        Ok(self)
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

        assert!(cache.cache().is_empty());
        let mut gg = gain.init(&geometry, None)?;
        let mut gc = cache.init(&geometry, None)?;
        geometry.devices().try_for_each(|dev| {
            let gf = gg.generate(dev);
            let cf = gc.generate(dev);
            dev.iter().try_for_each(|tr| {
                assert_eq!(gf.calc(tr), cf.calc(tr));
                Result::<(), AUTDInternalError>::Ok(())
            })
        })?;
        Ok(())
    }

    #[test]
    fn different_geometry() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2);

        let gain = Uniform::new(Drive::NULL);
        let cache = gain.with_cache();

        cache.clone().init(&geometry, None)?;

        geometry[1].enable = false;

        assert_eq!(
            Some(AUTDInternalError::GainError(
                "Cache is initialized with different geometry".to_string()
            )),
            cache.init(&geometry, None).err()
        );

        Ok(())
    }

    #[derive(Gain, Clone, self::Debug)]
    struct CacheTestGain {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    struct CacheTestGainContext {}

    impl GainContext for CacheTestGainContext {
        fn calc(&self, _: &Transducer) -> Drive {
            Drive::NULL
        }
    }

    impl GainContextGenerator for CacheTestGain {
        type Context = CacheTestGainContext;

        fn generate(&mut self, _: &Device) -> Self::Context {
            CacheTestGainContext {}
        }
    }

    impl Gain for CacheTestGain {
        type G = CacheTestGain;

        fn init(
            self,
            _geometry: &Geometry,
            _filter: Option<&HashMap<usize, BitVec<u32>>>,
        ) -> Result<Self::G, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(self)
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
        let _ = gain.clone().init(&geometry, None);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.init(&geometry, None);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }

    #[test]
    fn test_clone() {
        let geometry = create_geometry(1);

        {
            let calc_cnt = Arc::new(AtomicUsize::new(0));
            let gain = CacheTestGain {
                calc_cnt: calc_cnt.clone(),
            };

            assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

            let _ = gain.clone().init(&geometry, None);
            assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
            let _ = gain.init(&geometry, None);
            assert_eq!(2, calc_cnt.load(Ordering::Relaxed));
        }

        {
            let calc_cnt = Arc::new(AtomicUsize::new(0));
            let gain = CacheTestGain {
                calc_cnt: calc_cnt.clone(),
            }
            .with_cache();
            assert_eq!(1, gain.count());
            assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

            let g2 = gain.clone();
            assert_eq!(2, gain.count());
            assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

            let _ = g2.clone().init(&geometry, None);
            assert_eq!(2, gain.count());
            assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

            let _ = g2.init(&geometry, None);
            assert_eq!(1, gain.count());
            assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        }
    }
}
