use autd3_driver::{common::EmitIntensity, derive::*};

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

/// Modulation to cache the result of calculation
#[derive(Modulation)]
pub struct Cache<M: Modulation> {
    m: Rc<M>,
    cache: Rc<RefCell<Vec<EmitIntensity>>>,
    #[no_change]
    config: SamplingConfiguration,
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

impl<M: Modulation> IntoCache<M> for M {
    fn with_cache(self) -> Cache<M> {
        Cache::new(self)
    }
}

impl<M: Modulation + Clone> Clone for Cache<M> {
    fn clone(&self) -> Self {
        Self {
            m: self.m.clone(),
            cache: self.cache.clone(),
            config: self.config,
        }
    }
}

impl<M: Modulation> Cache<M> {
    /// constructor
    pub fn new(m: M) -> Self {
        Self {
            config: m.sampling_config(),
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
    /// let m = Static::new().with_cache();
    /// assert!(m.buffer().is_empty());
    /// let _ = m.calc().unwrap();
    /// assert!(!m.buffer().is_empty());
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
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::modulation::Static;

    use super::*;

    #[test]
    fn test_cache() {
        let m = Static::new().with_cache();
        assert_eq!(m.sampling_config(), Static::new().sampling_config());
        assert_eq!(m.intensity(), Static::new().intensity());

        assert!(m.buffer().is_empty());
        m.calc().unwrap().iter().for_each(|&d| {
            assert_eq!(d, EmitIntensity::MAX);
        });

        assert!(!m.buffer().is_empty());
        m.buffer().iter().for_each(|&d| {
            assert_eq!(d, EmitIntensity::MAX);
        });
    }

    struct TestModulation {
        pub calc_cnt: Arc<AtomicUsize>,
        pub config: SamplingConfiguration,
    }

    impl Clone for TestModulation {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn clone(&self) -> Self {
            Self {
                calc_cnt: self.calc_cnt.clone(),
                config: self.config,
            }
        }
    }

    impl ModulationProperty for TestModulation {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn sampling_config(&self) -> SamplingConfiguration {
            self.config
        }
    }

    impl Modulation for TestModulation {
        fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(vec![EmitIntensity::new(0); 2])
        }
    }

    #[test]
    fn test_cache_calc_once() {
        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfiguration::FREQ_4K_HZ,
        }
        .with_cache();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 0);

        let _ = modulation.calc().unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);

        let _ = modulation.calc().unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_cache_calc_clone() {
        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfiguration::FREQ_4K_HZ,
        }
        .with_cache();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 0);

        let m2 = modulation.clone();
        let _ = m2.calc().unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);

        assert_eq!(modulation.buffer().to_vec(), m2.buffer().to_vec());
    }
}
