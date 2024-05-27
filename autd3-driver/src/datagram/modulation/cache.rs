use crate::derive::*;

use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Modulation)]
#[no_modulation_cache]
#[no_radiation_pressure]
#[no_modulation_transform]
pub struct Cache<M: Modulation> {
    m: M,
    cache: Arc<RwLock<HashMap<usize, Vec<u8>>>>,
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
    pub fn new(m: M) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m,
            cache: Arc::new(Default::default()),
        }
    }

    pub fn init(&self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.cache.read().unwrap().is_empty() {
            let f = self.m.calc(geometry)?;
            *self.cache.write().unwrap() = geometry
                .devices()
                .map(|dev| (dev.idx(), { f(dev) }))
                .collect();
        }
        Ok(())
    }

    pub fn buffer(&self) -> RwLockReadGuard<HashMap<usize, Vec<u8>>> {
        self.cache.read().unwrap()
    }
}

impl<M: Modulation> Modulation for Cache<M> {
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        self.init(geometry)?;
        let buffer = self.buffer().clone();
        Ok(Box::new(move |dev| buffer[&dev.idx()].clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{defined::kHz, defined::FREQ_40K, geometry::tests::create_geometry};

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
        let geometry = create_geometry(1, 249, FREQ_40K);

        let mut rng = rand::thread_rng();

        let m = TestModulation {
            buf: vec![rng.gen(), rng.gen()],
            config: SamplingConfig::Freq(4 * kHz),
            loop_behavior: LoopBehavior::infinite(),
        };
        let cache = m.clone().with_cache();
        assert_eq!(&m, cache.deref());

        assert!(cache.buffer().is_empty());

        geometry.devices().try_for_each(|dev| {
            assert_eq!(m.calc(&geometry)?(dev), cache.calc(&geometry)?(dev));
            Result::<(), AUTDInternalError>::Ok(())
        })?;

        Ok(())
    }

    #[derive(Modulation, Clone)]
    struct TestCacheModulation {
        pub calc_cnt: Arc<AtomicUsize>,
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestCacheModulation {
        fn calc<'a>(&'a self, _: &'a Geometry) -> ModulationCalcResult {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Box::new(move |_| vec![0x00, 0x00]))
        }
    }

    #[test]
    fn test_calc_once() {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfig::Freq(4 * kHz),
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
        let geometry = create_geometry(1, 249, FREQ_40K);

        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let modulation = TestCacheModulation {
            calc_cnt: calc_cnt.clone(),
            config: SamplingConfig::Freq(4 * kHz),
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
