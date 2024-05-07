pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    firmware::operation::{GainOp, NullOp, Operation},
    geometry::Geometry,
};
pub use autd3_derive::Gain;

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

/// Gain to cache the result of calculation
#[derive(Gain, Debug)]
#[no_gain_cache]
#[no_gain_transform]
pub struct Cache<G: Gain + 'static> {
    gain: Rc<G>,
    cache: Rc<RefCell<HashMap<usize, Vec<Drive>>>>,
}

impl<G: Gain + 'static> std::ops::Deref for Cache<G> {
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.gain
    }
}

pub trait IntoCache<G: Gain + 'static> {
    /// Cache the result of calculation
    fn with_cache(self) -> Cache<G>;
}

impl<G: Gain + Clone + 'static> Clone for Cache<G> {
    fn clone(&self) -> Self {
        Self {
            gain: self.gain.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<G: Gain + PartialEq + 'static> PartialEq for Cache<G> {
    fn eq(&self, other: &Self) -> bool {
        self.gain == other.gain && self.cache == other.cache
    }
}

impl<G: Gain + 'static> Cache<G> {
    /// constructor
    #[doc(hidden)]
    pub fn new(gain: G) -> Self {
        Self {
            gain: Rc::new(gain),
            cache: Rc::new(Default::default()),
        }
    }

    pub fn init(&self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.cache.borrow().len() != geometry.devices().count()
            || geometry
                .devices()
                .any(|dev| !self.cache.borrow().contains_key(&dev.idx()))
        {
            *self.cache.borrow_mut() = self.gain.calc(geometry, GainFilter::All)?;
        }
        Ok(())
    }

    /// get cached drives
    ///
    /// Note that the cached data is created after at least one call to `calc`.
    pub fn drives(&self) -> Ref<'_, HashMap<usize, Vec<Drive>>> {
        self.cache.borrow()
    }
}

impl<G: Gain + 'static> Gain for Cache<G> {
    fn calc(
        &self,
        geometry: &Geometry,
        _filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        self.init(geometry)?;
        Ok(self.cache.borrow().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::TestGain, *};

    use rand::Rng;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::{defined::FREQ_40K, derive::*, geometry::tests::create_geometry};

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let mut rng = rand::thread_rng();
        let d: Drive = rng.gen();
        let gain = TestGain {
            f: move |_| move |_| d,
        };
        let cache = gain.with_cache();

        assert!(cache.drives().is_empty());
        assert_eq!(
            gain.calc(&geometry, GainFilter::All)?,
            cache.calc(&geometry, GainFilter::All)?
        );
        assert_eq!(gain.calc(&geometry, GainFilter::All)?, *cache.drives());

        Ok(())
    }

    #[derive(Gain, Clone, Debug)]
    pub struct CacheTestGain {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    impl PartialEq for CacheTestGain {
        fn eq(&self, other: &Self) -> bool {
            self.calc_cnt.load(Ordering::Relaxed) == other.calc_cnt.load(Ordering::Relaxed)
        }
    }

    impl Gain for CacheTestGain {
        fn calc(
            &self,
            geometry: &Geometry,
            filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Self::transform(geometry, filter, |_| |_| Drive::null()))
        }
    }

    #[test]
    fn test_calc_once() {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let calc_cnt = Arc::new(AtomicUsize::new(0));
        let gain = CacheTestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }

    #[test]
    fn test_clone() {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let calc_cnt = Arc::new(AtomicUsize::new(0));
        let gain = CacheTestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        let g2 = gain.clone();
        assert_eq!(gain, g2);
        assert_eq!(0, gain.calc_cnt.load(Ordering::Relaxed));
        assert_eq!(0, g2.calc_cnt.load(Ordering::Relaxed));
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let _ = g2.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = g2.calc(&geometry, GainFilter::All);
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
    }
}
