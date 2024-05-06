use crate::derive::*;

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

/// Modulation to cache the result of calculation
#[derive(Modulation)]
#[no_modulation_cache]
#[no_radiation_pressure]
#[no_modulation_transform]
pub struct Cache<M: Modulation> {
    m: Rc<M>,
    cache: Rc<RefCell<HashMap<usize, Vec<u8>>>>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation> std::ops::Deref for Cache<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.m
    }
}

pub trait IntoCache<M: Modulation> {
    /// Cache the result of calculation
    fn with_cache(self) -> Cache<M>;
}

impl<M: Modulation + Clone> Clone for Cache<M> {
    fn clone(&self) -> Self {
        Self {
            m: self.m.clone(),
            cache: self.cache.clone(),
            config: self.config,
            loop_behavior: self.loop_behavior,
        }
    }
}

impl<M: Modulation> Cache<M> {
    /// constructor
    pub fn new(m: M) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m: Rc::new(m),
            cache: Rc::new(Default::default()),
        }
    }

    /// get cached modulation data
    ///
    /// Note that the cached data is created after at least one call to `calc`.
    pub fn buffer(&self) -> Ref<'_, HashMap<usize, Vec<u8>>> {
        self.cache.borrow()
    }
}

impl<M: Modulation> Modulation for Cache<M> {
    fn calc(&self, geometry: &Geometry) -> Result<HashMap<usize, Vec<u8>>, AUTDInternalError> {
        if self.cache.borrow().is_empty() {
            *self.cache.borrow_mut() = self.m.calc(geometry)?;
        }
        Ok(self.cache.borrow().clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::tests::create_geometry;

    use super::{super::tests::TestModulation, *};

    use rand::Rng;
    use std::{
        ops::Deref,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249);

        let mut rng = rand::thread_rng();

        let m = TestModulation {
            buf: vec![rng.gen(), rng.gen()],
            config: SamplingConfig::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::infinite(),
        };
        let cache = m.clone().with_cache();
        assert_eq!(&m, cache.deref());

        assert!(cache.buffer().is_empty());
        assert_eq!(m.calc(&geometry)?, cache.calc(&geometry)?);

        assert!(!cache.buffer().is_empty());
        assert_eq!(m.calc(&geometry)?, *cache.buffer());

        Ok(())
    }

    #[derive(Modulation)]
    struct TestCacheModulation {
        pub calc_cnt: Arc<AtomicUsize>,
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Clone for TestCacheModulation {
        // GRCOV_EXCL_START
        fn clone(&self) -> Self {
            Self {
                calc_cnt: self.calc_cnt.clone(),
                config: self.config,
                loop_behavior: LoopBehavior::infinite(),
            }
        }
        // GRCOV_EXCL_STOP
    }

    impl Modulation for TestCacheModulation {
        fn calc(&self, geometry: &Geometry) -> Result<HashMap<usize, Vec<u8>>, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Self::transform(geometry, |_| Ok(vec![0; 2]))
        }
    }

    #[test]
    fn test_calc_once() {
        let geometry = create_geometry(1, 249);

        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfig::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_cache();
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let _ = modulation.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        let _ = modulation.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }

    #[test]
    fn test_calc_clone() {
        let geometry = create_geometry(1, 249);

        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfig::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_cache();
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let m2 = modulation.clone();
        let _ = m2.calc(&geometry);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        assert_eq!(*modulation.buffer(), *m2.buffer());
    }
}
