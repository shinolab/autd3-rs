use autd3_core::derive::*;
use autd3_derive::Builder;

use std::{cell::RefCell, rc::Rc};

use derive_more::Debug;

/// Cache for [`Modulation`]
#[derive(Modulation, Debug, Builder)]
pub struct Cache<M: Modulation> {
    m: Rc<RefCell<Option<M>>>,
    #[debug("{}", !self.cache.borrow().is_empty())]
    #[get]
    /// Cached modulation data.
    cache: Rc<RefCell<Vec<u8>>>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation> Clone for Cache<M> {
    fn clone(&self) -> Self {
        Self {
            m: self.m.clone(),
            cache: self.cache.clone(),
            config: self.config,
            loop_behavior: self.loop_behavior,
        }
    }
}

/// Trait to convert [`Modulation`] to [`Cache`].
pub trait IntoCache<M: Modulation> {
    /// Convert [`Modulation`] to [`Cache`]
    fn with_cache(self) -> Cache<M>;
}

impl<M: Modulation> IntoCache<M> for M {
    fn with_cache(self) -> Cache<M> {
        Cache::new(self)
    }
}

impl<M: Modulation> Cache<M> {
    fn new(m: M) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m: Rc::new(RefCell::new(Some(m))),
            cache: Rc::default(),
        }
    }

    /// Initialize cache.
    pub fn init(&self) -> Result<(), ModulationError> {
        if let Some(m) = self.m.take() {
            tracing::debug!("Initializing cache");
            *self.cache.borrow_mut() = m.calc()?;
        }
        Ok(())
    }

    /// Get the number of references to the cache.
    pub fn count(&self) -> usize {
        Rc::strong_count(&self.cache)
    }
}

impl<M: Modulation> Modulation for Cache<M> {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        self.init()?;
        let buffer = self.cache().clone();
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::modulation::Custom;

    use super::*;

    use rand::Rng;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let m = Custom::new([rng.gen::<u8>(), rng.gen::<u8>()], SamplingConfig::FREQ_4K)?;
        let cache = m.clone().with_cache();

        assert!(cache.cache().is_empty());

        assert_eq!(m.calc()?, cache.calc()?);

        Ok(())
    }

    #[derive(Modulation, Clone, self::Debug)]
    struct TestCacheModulation {
        pub calc_cnt: Arc<AtomicUsize>,
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestCacheModulation {
        fn calc(self) -> Result<Vec<u8>, ModulationError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(vec![0x00, 0x00])
        }
    }

    #[test]
    fn test_calc_once() {
        {
            let calc_cnt = Arc::new(AtomicUsize::new(0));

            let modulation = TestCacheModulation {
                calc_cnt: calc_cnt.clone(),
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::infinite(),
            };
            assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

            let _ = modulation.clone().calc();
            assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

            let _ = modulation.calc();
            assert_eq!(2, calc_cnt.load(Ordering::Relaxed));
        }

        {
            let calc_cnt = Arc::new(AtomicUsize::new(0));

            let modulation = TestCacheModulation {
                calc_cnt: calc_cnt.clone(),
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_cache();
            assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

            let _ = modulation.clone().calc();
            assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

            let _ = modulation.calc();
            assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn test_calc_clone() {
        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_cache();
        assert_eq!(1, modulation.count());
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let m2 = modulation.clone();
        assert_eq!(2, modulation.count());
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let _ = m2.clone().calc();
        assert_eq!(2, modulation.count());
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        assert_eq!(*modulation.cache(), *m2.cache());

        let _ = m2.calc();
        assert_eq!(1, modulation.count());
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }
}
