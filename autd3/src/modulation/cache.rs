/*
 * File: Cache.rs
 * Project: gain
 * Created Date: 10/05/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 01/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_derive::Modulation;
use autd3_driver::{common::EmitIntensity, derive::prelude::*};

use std::ops::Deref;

/// Modulation to cache the result of calculation
#[derive(Modulation)]
pub struct Cache {
    cache: Vec<EmitIntensity>,
    #[no_change]
    config: SamplingConfiguration,
}

pub trait IntoCache<M: Modulation> {
    /// Cache the result of calculation
    fn with_cache(self) -> Result<Cache, AUTDInternalError>;
}

impl<M: Modulation> IntoCache<M> for M {
    fn with_cache(self) -> Result<Cache, AUTDInternalError> {
        Cache::new(self)
    }
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            config: self.config,
        }
    }
}

impl Cache {
    /// constructor
    pub fn new<M: Modulation>(modulation: M) -> Result<Self, AUTDInternalError> {
        let config = modulation.sampling_config();
        Ok(Self {
            cache: modulation.calc()?,
            config,
        })
    }

    /// get cached modulation data
    pub fn buffer(&self) -> &[EmitIntensity] {
        &self.cache
    }
}

impl Modulation for Cache {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(self.cache.clone())
    }
}

impl Deref for Cache {
    type Target = [EmitIntensity];

    fn deref(&self) -> &Self::Target {
        &self.cache
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

    use autd3_derive::Modulation;

    #[test]
    fn test_cache() {
        let m = Static::new().with_cache().unwrap();

        for d in m.calc().unwrap() {
            assert_eq!(d.value(), 0xFF);
        }
    }

    #[derive(Modulation)]
    struct TestModulation {
        pub calc_cnt: Arc<AtomicUsize>,
        pub config: SamplingConfiguration,
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
            config: SamplingConfiguration::from_period(std::time::Duration::from_micros(250))
                .unwrap(),
        }
        .with_cache()
        .unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);

        let _ = modulation.calc().unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
        let _ = modulation.calc().unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
        let _ = modulation.calc().unwrap();
        assert_eq!(calc_cnt.load(Ordering::Relaxed), 1);
    }
}
