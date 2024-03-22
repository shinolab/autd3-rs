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
    cache: Rc<RefCell<Vec<EmitIntensity>>>,
    #[no_change]
    config: SamplingConfiguration,
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
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3::prelude::*;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let m = Static::new().with_cache();
    /// assert!(m.buffer().is_empty());
    /// let _ = m.calc()?;
    /// assert!(!m.buffer().is_empty());
    /// # Ok(())
    /// # }
    ///
    /// ```
    pub fn buffer(&self) -> Ref<'_, Vec<EmitIntensity>> {
        self.cache.borrow()
    }
}

impl<M: Modulation> Modulation for Cache<M> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        if self.cache.borrow().is_empty() {
            *self.cache.borrow_mut() = self.m.calc()?;
        }
        Ok(self.cache.borrow().clone())
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
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let m = TestModulation {
            buf: vec![rng.gen(), rng.gen()],
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
        };
        let cache = m.clone().with_cache();
        assert_eq!(&m, cache.deref());

        assert!(cache.buffer().is_empty());
        assert_eq!(m.calc()?, cache.calc()?);

        assert!(!cache.buffer().is_empty());
        assert_eq!(m.calc()?, *cache.buffer());

        Ok(())
    }

    #[derive(Modulation)]
    struct TestCacheModulation {
        pub calc_cnt: Arc<AtomicUsize>,
        pub config: SamplingConfiguration,
        pub loop_behavior: LoopBehavior,
    }

    impl Clone for TestCacheModulation {
        // GRCOV_EXCL_START
        fn clone(&self) -> Self {
            Self {
                calc_cnt: self.calc_cnt.clone(),
                config: self.config,
                loop_behavior: LoopBehavior::Infinite,
            }
        }
        // GRCOV_EXCL_STOP
    }

    impl Modulation for TestCacheModulation {
        fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(vec![EmitIntensity::new(0); 2])
        }
    }

    #[test]
    fn test_calc_once() {
        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
        }
        .with_cache();
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let _ = modulation.calc();
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        let _ = modulation.calc();
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }

    #[test]
    fn test_calc_clone() {
        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
        }
        .with_cache();
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let m2 = modulation.clone();
        let _ = m2.calc();
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        assert_eq!(*modulation.buffer(), *m2.buffer());
    }
}
