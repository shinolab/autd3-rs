/*
 * File: Cache.rs
 * Project: gain
 * Created Date: 10/05/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_derive::Gain;
use autd3_driver::{derive::prelude::*, geometry::Geometry};

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

/// Gain to cache the result of calculation
#[derive(Gain)]
pub struct Cache<G: Gain + 'static> {
    gain: Rc<G>,
    cache: Rc<RefCell<HashMap<usize, Vec<Drive>>>>,
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
            self.gain
                .calc(geometry, GainFilter::All)?
                .iter()
                .for_each(|(k, v)| {
                    self.cache.borrow_mut().insert(*k, v.clone());
                });
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
    /// # use autd3_driver::derive::prelude::GainFilter;
    /// # use autd3_driver::datagram::Gain;
    /// # let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
    ///
    /// let g = Null::new().with_cache();
    /// assert!(g.drives().is_empty());
    /// let _ = g.calc(&geometry, GainFilter::All).unwrap();
    /// assert!(!g.drives().is_empty());
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

    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{IntoDevice, Vector3},
    };

    use super::{
        super::{Null, Plane},
        *,
    };

    #[test]
    fn test_cache() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let gain = Plane::new(Vector3::zeros()).with_cache();

        let d = gain.calc(&geometry, GainFilter::All).unwrap();
        d[&0].iter().for_each(|drive| {
            assert_eq!(drive.phase.value(), 0);
            assert_eq!(drive.intensity.value(), 0xFF);
        });
        gain.drives().iter().for_each(|(k, v)| {
            assert_eq!(k, &0);
            v.iter().for_each(|drive| {
                assert_eq!(drive.phase.value(), 0);
                assert_eq!(drive.intensity.value(), 0xFF);
            });
        });
    }

    struct TestGain {
        pub calc_cnt: Arc<AtomicUsize>,
    }

    impl Clone for TestGain {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn clone(&self) -> Self {
            Self {
                calc_cnt: self.calc_cnt.clone(),
            }
        }
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
    fn test_cache_calc_once() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let gain = TestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        assert_eq!(calc_cnt.load(Ordering::Relaxed), 0);
        let _ = gain.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
        let _ = gain.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_cache_clone() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let calc_cnt = Arc::new(AtomicUsize::new(0));

        let gain = TestGain {
            calc_cnt: calc_cnt.clone(),
        }
        .with_cache();

        let g2 = gain.clone();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 0);

        let _ = g2.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
        let _ = gain.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
        let _ = g2.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_cache_derive() {
        let g = Null::default().with_cache();
        let _ = g.operation();
    }
}
