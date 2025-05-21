use autd3_core::derive::*;

use std::{cell::RefCell, rc::Rc};

use derive_more::Debug;
use getset::Getters;

/// Cache for [`Gain`]
///
/// This [`Gain`] is used to cache the calculated phases and intensities for each transducer.
#[derive(Gain, Debug, Getters)]
pub struct Cache<G: Gain> {
    gain: Rc<RefCell<Option<G>>>,
    #[getset(get = "pub")]
    #[debug("{}", !self.cache.borrow().is_empty())]
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

impl<G: Gain> Cache<G> {
    /// Create a new cached [`Gain`].
    #[must_use]
    pub fn new(gain: G) -> Self {
        Self {
            gain: Rc::new(RefCell::new(Some(gain))),
            cache: Default::default(),
        }
    }

    /// Initialize cache
    ///
    /// # Errors
    ///
    /// Returns [`GainError`] if you initialize with some devices disabled and then reinitialize after enabling the devices.
    pub fn init(
        &self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<(), GainError> {
        if let Some(gain) = self.gain.take() {
            let mut f = gain.init(geometry, filter)?;
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
            return Err(GainError::new(
                "Cache is initialized with different geometry",
            ));
        }

        Ok(())
    }

    /// Get the number of references to the cache
    #[must_use]
    pub fn count(&self) -> usize {
        Rc::strong_count(&self.cache)
    }
}

pub struct Impl {
    g: Arc<Vec<Drive>>,
}

impl GainCalculator for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.g[tr.idx()]
    }
}

impl<G: Gain> GainCalculatorGenerator for Cache<G> {
    type Calculator = Impl;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        Impl {
            g: self.cache.borrow()[&device.idx()].clone(),
        }
    }
}

impl<G: Gain> Gain for Cache<G> {
    type G = Self;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::G, GainError> {
        Cache::init(&self, geometry, filter)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{gain::Uniform, tests::create_geometry};

    use super::*;

    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;
    use std::{
        fmt::Debug,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let mut rng = rand::rng();
        let d = Drive {
            phase: Phase(rng.random()),
            intensity: EmitIntensity(rng.random()),
        };
        let gain = Uniform {
            intensity: d.intensity,
            phase: d.phase,
        };
        let cache = Cache::new(gain.clone());

        assert!(cache.cache().borrow().is_empty());
        let mut gg = gain.init(&geometry, None)?;
        let mut gc = cache.init(&geometry, None)?;
        geometry.devices().try_for_each(|dev| {
            let gf = gg.generate(dev);
            let cf = gc.generate(dev);
            dev.iter().try_for_each(|tr| {
                assert_eq!(gf.calc(tr), cf.calc(tr));
                Result::<(), GainError>::Ok(())
            })
        })?;
        Ok(())
    }

    #[test]
    fn different_geometry() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2);

        let gain = Uniform {
            intensity: EmitIntensity::MIN,
            phase: Phase::ZERO,
        };
        let cache = Cache::new(gain);

        cache.clone().init(&geometry, None)?;

        geometry[1].enable = false;

        assert_eq!(
            Some(GainError::new(
                "Cache is initialized with different geometry"
            )),
            cache.init(&geometry, None).err()
        );

        Ok(())
    }

    #[derive(Gain, Clone, self::Debug)]
    struct CacheTestGain {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    struct CacheTestGainCalculator {}

    impl GainCalculator for CacheTestGainCalculator {
        fn calc(&self, _: &Transducer) -> Drive {
            Drive::NULL
        }
    }

    impl GainCalculatorGenerator for CacheTestGain {
        type Calculator = CacheTestGainCalculator;

        fn generate(&mut self, _: &Device) -> Self::Calculator {
            CacheTestGainCalculator {}
        }
    }

    impl Gain for CacheTestGain {
        type G = CacheTestGain;

        fn init(
            self,
            _: &Geometry,
            _: Option<&HashMap<usize, BitVec>>,
        ) -> Result<Self::G, GainError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(self)
        }
    }

    #[test]
    fn test_calc_once() {
        let geometry = create_geometry(1);

        let calc_cnt = Arc::new(AtomicUsize::new(0));
        let gain = Cache::new(CacheTestGain {
            calc_cnt: calc_cnt.clone(),
        });

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
            let gain = Cache::new(CacheTestGain {
                calc_cnt: calc_cnt.clone(),
            });
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
