pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    firmware::operation::{GainOp, NullOp, Operation},
    geometry::{Device, Geometry, Transducer},
};
pub use autd3_derive::Gain;

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use bitvec::prelude::*;

#[derive(Gain)]
pub struct Group<K, F>
where
    K: Hash + Eq + Clone + Debug + 'static,
    F: Fn(&Device, &Transducer) -> Option<K> + 'static,
{
    f: F,
    gain_map: HashMap<K, Box<dyn Gain>>,
}

impl<K, F> Group<K, F>
where
    K: Hash + Eq + Clone + Debug + 'static,
    F: Fn(&Device, &Transducer) -> Option<K> + 'static,
{
    /// Group by transducer
    ///
    /// # Arguments
    ///
    /// `f` - function to get key
    pub fn new(f: F) -> Group<K, F> {
        Group {
            f,
            gain_map: HashMap::new(),
        }
    }

    /// set gain
    ///
    /// # Arguments
    ///
    /// * `key` - key
    /// * `gain` - Gain
    ///
    pub fn set(mut self, key: K, gain: impl Gain + 'static) -> Self {
        self.gain_map.insert(key, Box::new(gain));
        self
    }

    fn get_filters(&self, geometry: &Geometry) -> HashMap<K, HashMap<usize, BitVec<usize, Lsb0>>> {
        let mut filters: HashMap<K, HashMap<usize, BitVec<usize, Lsb0>>> = HashMap::new();
        geometry.devices().for_each(|dev| {
            dev.iter().for_each(|tr| {
                if let Some(key) = (self.f)(dev, tr) {
                    match filters.get_mut(&key) {
                        Some(v) => match v.entry(dev.idx()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().set(tr.idx(), true);
                            }
                            Entry::Vacant(e) => {
                                let mut filter =
                                    BitVec::<usize, Lsb0>::repeat(false, dev.num_transducers());
                                filter.set(tr.idx(), true);
                                e.insert(filter);
                            }
                        },
                        None => {
                            let mut filter =
                                BitVec::<usize, Lsb0>::repeat(false, dev.num_transducers());
                            filter.set(tr.idx(), true);
                            filters.insert(key.clone(), [(dev.idx(), filter)].into());
                        }
                    }
                }
            })
        });
        filters
    }
}

impl<K, F> Gain for Group<K, F>
where
    K: Hash + Eq + Clone + Debug + 'static,
    F: Fn(&Device, &Transducer) -> Option<K> + 'static,
{
    fn calc(
        &self,
        geometry: &Geometry,
        _filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let filters = self.get_filters(geometry);

        let specified_keys = self.gain_map.keys().cloned().collect::<HashSet<_>>();
        let provided_keys = filters.keys().cloned().collect::<HashSet<_>>();

        let unknown_keys = specified_keys
            .difference(&provided_keys)
            .collect::<Vec<_>>();
        if !unknown_keys.is_empty() {
            return Err(AUTDInternalError::UnkownKey(format!("{:?}", unknown_keys)));
        }
        let unspecified_keys = provided_keys
            .difference(&specified_keys)
            .collect::<Vec<_>>();
        if !unspecified_keys.is_empty() {
            return Err(AUTDInternalError::UnspecifiedKey(format!(
                "{:?}",
                unspecified_keys
            )));
        }

        let drives_cache = self
            .gain_map
            .iter()
            .map(|(k, g)| {
                Ok((
                    k.clone(),
                    g.calc(geometry, GainFilter::Filter(&filters[&k]))?,
                ))
            })
            .collect::<Result<HashMap<_, _>, AUTDInternalError>>()?;
        geometry
            .devices()
            .map(|dev| {
                Ok((
                    dev.idx(),
                    dev.iter()
                        .map(|tr| {
                            (self.f)(dev, tr).map_or_else(Drive::null, |key| {
                                drives_cache[&key][&dev.idx()][tr.idx()]
                            })
                        })
                        .collect::<Vec<_>>(),
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestGain, *};

    use crate::{firmware::operation::tests::NullGain, geometry::tests::create_geometry};

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(4, 249);

        let mut rng = rand::thread_rng();

        let d1: Drive = rng.gen();
        let d2: Drive = rng.gen();

        let g1 = TestGain {
            f: move |_| move |_| d1,
        };
        let g2 = TestGain {
            f: move |_| move |_| d2,
        };

        let gain = Group::new(|dev, tr| match (dev.idx(), tr.idx()) {
            (0, 0..=99) => Some("null"),
            (0, 100..=199) => Some("test"),
            (1, 200..) => Some("test2"),
            (3, _) => Some("test"),
            _ => None,
        })
        .set("null", NullGain {})
        .set("test", g1)
        .set("test2", g2);

        let drives = gain.calc(&geometry, GainFilter::All)?;
        assert_eq!(4, drives.len());
        drives[&0].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 99 => {
                assert_eq!(Drive::null(), d);
            }
            i if i <= 199 => {
                assert_eq!(d1, d);
            }
            _ => {
                assert_eq!(Drive::null(), d);
            }
        });
        drives[&1].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 199 => {
                assert_eq!(Drive::null(), d);
            }
            _ => {
                assert_eq!(d2, d);
            }
        });
        drives[&2].iter().for_each(|&d| {
            assert_eq!(Drive::null(), d);
        });
        drives[&3].iter().for_each(|&d| {
            assert_eq!(d1, d);
        });

        Ok(())
    }

    #[test]
    fn test_unknown_key() {
        let geometry = create_geometry(2, 249);

        let gain = Group::new(|_dev, tr| match tr.idx() {
            0..=99 => Some("test"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("test2", NullGain {});

        assert_eq!(
            Err(AUTDInternalError::UnkownKey("[\"test2\"]".to_owned())),
            gain.calc(&geometry, GainFilter::All)
        );
    }

    #[test]
    fn test_unspecified_key() {
        let geometry = create_geometry(2, 249);

        let gain = Group::new(|_dev, tr| match tr.idx() {
            0..=99 => Some("test"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("test", NullGain {});

        assert_eq!(
            Err(AUTDInternalError::UnspecifiedKey("[\"null\"]".to_owned())),
            gain.calc(&geometry, GainFilter::All)
        );
    }

    #[derive(Gain, Clone, Copy, PartialEq, Debug)]
    pub struct ErrGain {}

    impl Gain for ErrGain {
        fn calc(
            &self,
            _geometry: &Geometry,
            _filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            Err(AUTDInternalError::GainError("test error".to_owned()))
        }
    }

    #[test]
    fn test_calc_err() {
        let geometry = create_geometry(2, 249);

        let gain = Group::new(|_dev, tr| match tr.idx() {
            _ => Some("test"),
        })
        .set("test", ErrGain {});

        assert_eq!(
            Err(AUTDInternalError::GainError("test error".to_owned())),
            gain.calc(&geometry, GainFilter::All)
        );
    }
}
