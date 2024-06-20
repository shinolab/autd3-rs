use crate::derive::*;

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use derive_more::Deref;

#[derive(Modulation, Clone, Deref)]
#[no_modulation_cache]
#[no_radiation_pressure]
#[no_modulation_transform]
pub struct Cache<M: Modulation> {
    #[deref]
    m: M,
    cache: Rc<RefCell<Vec<u8>>>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

pub trait IntoCache<M: Modulation> {
    fn with_cache(self) -> Cache<M>;
}

impl<M: Modulation> Cache<M> {
    pub fn new(m: M) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m,
            cache: Rc::default(),
        }
    }

    pub fn init(&self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.cache.borrow().is_empty() {
            *self.cache.borrow_mut() = self.m.calc(geometry)?;
        }
        Ok(())
    }

    pub fn buffer(&self) -> Ref<'_, Vec<u8>> {
        self.cache.borrow()
    }
}

impl<M: Modulation> Modulation for Cache<M> {
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        self.init(geometry)?;
        let buffer = self.buffer().clone();
        Ok(buffer)
    }

    #[tracing::instrument(level = "debug", skip(self, geometry), fields(%self.config, %self.loop_behavior, cached = !self.cache.borrow().is_empty()))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        <M as Modulation>::trace(&self.m, geometry);
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use crate::{defined::kHz, geometry::tests::create_geometry};

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
            config: SamplingConfig::Freq(4 * kHz),
            loop_behavior: LoopBehavior::infinite(),
        };
        let cache = m.clone().with_cache();
        assert_eq!(&m, cache.deref());

        assert!(cache.buffer().is_empty());

        assert_eq!(m.calc(&geometry)?, cache.calc(&geometry)?);

        Ok(())
    }

    #[derive(Modulation, Clone)]
    struct TestCacheModulation {
        pub calc_cnt: Arc<AtomicUsize>,
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestCacheModulation {
        fn calc(&self, _: &Geometry) -> ModulationCalcResult {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(vec![0x00, 0x00])
        }
    }

    #[test]
    fn test_calc_once() {
        let geometry = create_geometry(1, 249);

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
        let geometry = create_geometry(1, 249);

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
