use autd3_core::derive::*;

use std::{cell::RefCell, rc::Rc};

use derive_more::Debug;
use getset::Getters;

/// Cache for [`Modulation`]
#[derive(Modulation, Debug, Clone, Getters)]
pub struct Cache<M: Modulation> {
    m: Rc<RefCell<Option<M>>>,
    #[getset(get = "pub")]
    #[debug("{}", !self.cache.borrow().is_empty())]
    /// Cached modulation data.
    cache: Rc<RefCell<Vec<u8>>>,
}

/// Trait to convert [`Modulation`] to [`Cache`].
pub trait IntoCache<M: Modulation> {
    /// Convert [`Modulation`] to [`Cache`]
    fn into_cached(self) -> Cache<M>;
}

impl<M: Modulation> IntoCache<M> for M {
    fn into_cached(self) -> Cache<M> {
        Cache::new(self)
    }
}

impl<M: Modulation> Cache<M> {
    fn new(m: M) -> Self {
        Self {
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
    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        self.m.borrow().as_ref().unwrap().sampling_config()
    }

    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        self.init()?;
        let buffer = self.cache.borrow().clone();
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {

    use crate::modulation::Custom;

    use super::*;

    use rand::Rng;
    use std::{
        fmt::Debug,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let m = Custom {
            buffer: vec![rng.gen(), rng.gen()],
            sampling_config: SamplingConfig::DIV_10,
            option: Default::default(),
        };
        let cache = m.clone().into_cached();

        assert!(cache.cache().borrow().is_empty());

        assert_eq!(m.sampling_config(), cache.sampling_config());
        assert_eq!(m.calc()?, cache.calc()?);

        Ok(())
    }

    #[derive(Modulation, Clone, Debug)]
    struct TestCacheModulation {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    impl Modulation for TestCacheModulation {
        fn calc(self) -> Result<Vec<u8>, ModulationError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(vec![0x00, 0x00])
        }

        // GRCOV_EXCL_START
        fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
            unimplemented!()
        }
        // GRCOV_EXCL_STOP
    }

    #[test]
    fn test_calc_once() {
        {
            let calc_cnt = Arc::new(AtomicUsize::new(0));

            let modulation = TestCacheModulation {
                calc_cnt: calc_cnt.clone(),
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
            }
            .into_cached();
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
        }
        .into_cached();
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
