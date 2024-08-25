use crate::derive::*;

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
    sync::Arc,
};

use derive_more::{Debug, Deref};

#[derive(Modulation, Clone, Deref, Debug)]
pub struct Cache<M: Modulation> {
    #[deref]
    m: M,
    #[debug("{}", !self.cache.borrow().is_empty())]
    cache: Rc<RefCell<Arc<Vec<u8>>>>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

pub trait IntoCache<M: Modulation> {
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
            m,
            cache: Rc::default(),
        }
    }

    pub fn init(&self) -> Result<(), AUTDInternalError> {
        if self.cache.borrow().is_empty() {
            *self.cache.borrow_mut() = self.m.calc()?;
        }
        Ok(())
    }

    pub fn buffer(&self) -> Ref<'_, Arc<Vec<u8>>> {
        self.cache.borrow()
    }
}

impl<M: Modulation> Modulation for Cache<M> {
    fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
        self.init()?;
        let buffer = self.buffer().clone();
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
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
    #[cfg_attr(miri, ignore)]
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let m = TestModulation {
            buf: Arc::new(vec![rng.gen(), rng.gen()]),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        };
        let cache = m.clone().with_cache();
        assert_eq!(&m, cache.deref());

        assert!(cache.buffer().is_empty());

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
        fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Arc::new(vec![0x00, 0x00]))
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_calc_once() {
        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_cache();
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let _ = modulation.calc();
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        let _ = modulation.calc();
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_calc_clone() {
        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
        .with_cache();
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let m2 = modulation.clone();
        let _ = m2.calc();
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        assert_eq!(*modulation.buffer(), *m2.buffer());
    }
}
