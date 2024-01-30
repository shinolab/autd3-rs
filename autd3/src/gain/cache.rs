use autd3_driver::{derive::*, geometry::Geometry};

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

/// Gain to cache the result of calculation
#[derive(Gain, Debug)]
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

impl<G: Gain + 'static> IntoCache<G> for G {
    fn with_cache(self) -> Cache<G> {
        Cache::new(self)
    }
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
    fn new(gain: G) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3::prelude::*;
    /// # use autd3_driver::datagram::Gain;
    /// # fn main() -> anyhow::Result<()>{
    /// # let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
    /// let g = Null::new().with_cache();
    /// assert!(g.drives().is_empty());
    /// let _ = g.calc(&geometry, GainFilter::All)?;
    /// assert!(!g.drives().is_empty());
    /// # Ok(())
    /// # }
    ///
    /// ```
    pub fn drives(&self) -> Ref<'_, HashMap<usize, Vec<autd3_driver::common::Drive>>> {
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
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use autd3_driver::geometry::Vector3;

    use crate::tests::create_geometry;

    use super::{
        super::{Null, Plane},
        *,
    };

    #[test]
    fn test_cache() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let gain = Plane::new(Vector3::zeros());
        let cache = gain.with_cache();
        assert_eq!(gain.phase(), cache.phase());

        assert!(cache.drives().is_empty());
        assert_eq!(
            gain.calc(&geometry, GainFilter::All)?,
            cache.calc(&geometry, GainFilter::All)?
        );
        assert_eq!(gain.calc(&geometry, GainFilter::All)?, *cache.drives());

        Ok(())
    }

    #[derive(Clone)]
    struct TestGain {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    impl Gain for TestGain {
        fn calc(
            &self,
            geometry: &Geometry,
            filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            self.calc_cnt.fetch_add(1, Ordering::Relaxed);
            Ok(Self::transform(geometry, filter, |_, _| Drive::null()))
        }
    }

    #[test]
    fn test_cache_calc_once() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let calc_cnt = Arc::new(AtomicUsize::new(0));
        let gain = TestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All)?;
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All)?;
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        Ok(())
    }

    #[test]
    fn test_cache_clone() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let calc_cnt = Arc::new(AtomicUsize::new(0));
        let gain = TestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        let g2 = gain.clone();
        assert_eq!(0, calc_cnt.load(Ordering::Relaxed));

        let _ = g2.calc(&geometry, GainFilter::All)?;
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = gain.calc(&geometry, GainFilter::All)?;
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));
        let _ = g2.calc(&geometry, GainFilter::All)?;
        assert_eq!(1, calc_cnt.load(Ordering::Relaxed));

        Ok(())
    }

    #[test]
    fn test_cache_derive() {
        let g = Null::default().with_cache();
        assert_eq!(g, g.clone());
        let _ = g.operation();
    }
}
