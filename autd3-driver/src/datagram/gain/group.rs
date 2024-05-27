pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    firmware::operation::{GainOp, NullOp},
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
pub struct Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync + 'static,
    F: Fn(&Device) -> FK + Clone + Send + Sync + 'static,
{
    f: F,
    gain_map: HashMap<K, Box<dyn Gain>>,
}

impl<K, FK, F> Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync + 'static,
    F: Fn(&Device) -> FK + Clone + Send + Sync + 'static,
{
    pub fn new(f: F) -> Group<K, FK, F> {
        Group {
            f,
            gain_map: HashMap::new(),
        }
    }

    pub fn set(mut self, key: K, gain: impl Gain + 'static) -> Self {
        self.gain_map.insert(key, Box::new(gain));
        self
    }

    fn get_filters(&self, geometry: &Geometry) -> HashMap<K, HashMap<usize, BitVec<usize, Lsb0>>> {
        let mut filters: HashMap<K, HashMap<usize, BitVec<usize, Lsb0>>> = HashMap::new();
        geometry.devices().for_each(|dev| {
            dev.iter().for_each(|tr| {
                if let Some(key) = (self.f)(dev)(tr) {
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

impl<K, FK, F> Gain for Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync + 'static,
    F: Fn(&Device) -> FK + Clone + Send + Sync + 'static,
{
    fn calc(
        &self,
        geometry: &Geometry,
    ) -> Result<Box<dyn Fn(&Device) -> Vec<Drive> + Send + Sync>, AUTDInternalError> {
        let mut filters = self.get_filters(geometry);

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
                Ok((k.clone(), {
                    let f = g.calc_with_filter(geometry, filters.remove(k).unwrap())?;
                    geometry
                        .devices()
                        .map(move |dev| (dev.idx(), f(dev)))
                        .collect::<HashMap<_, _>>()
                }))
            })
            .collect::<Result<HashMap<_, _>, AUTDInternalError>>()?;

        let f = self.f.clone();
        Ok(Box::new(move |dev| {
            let fk = f(dev);
            let dev_idx = dev.idx();
            dev.iter()
                .map(|tr| {
                    fk(tr)
                        .map(|key| drives_cache[&key][&dev_idx][tr.idx()])
                        .unwrap_or(Drive::null())
                })
                .collect()
        }))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestGain, *};

    use crate::{defined::FREQ_40K, geometry::tests::create_geometry};

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(4, 249, FREQ_40K);

        let mut rng = rand::thread_rng();

        let d1: Drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let d2: Drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let g1 = TestGain::new(|_| |_| d1, &geometry);
        let g2 = TestGain::new(|_| |_| d2, &geometry);

        let gain = Group::new(|dev| {
            let dev_idx = dev.idx();
            move |tr| match (dev_idx, tr.idx()) {
                (0, 0..=99) => Some("null"),
                (0, 100..=199) => Some("test"),
                (1, 200..) => Some("test2"),
                (3, _) => Some("test"),
                _ => None,
            }
        })
        .set("null", TestGain::null(&geometry))
        .set("test", g1)
        .set("test2", g2);

        let g = gain.calc(&geometry)?;
        let drives = geometry
            .devices()
            .map(|dev| (dev.idx(), g(dev)))
            .collect::<HashMap<_, _>>();
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
        let geometry = create_geometry(2, 249, FREQ_40K);

        let gain = Group::new(|_dev| {
            |tr| match tr.idx() {
                0..=99 => Some("test"),
                100..=199 => Some("null"),
                _ => None,
            }
        })
        .set("test2", TestGain::null(&geometry));

        assert_eq!(
            Some(AUTDInternalError::UnkownKey("[\"test2\"]".to_owned())),
            gain.calc(&geometry).err()
        );
    }

    #[test]
    fn test_unspecified_key() {
        let geometry = create_geometry(2, 249, FREQ_40K);

        let gain = Group::new(|_dev| {
            |tr| match tr.idx() {
                0..=99 => Some("test"),
                100..=199 => Some("null"),
                _ => None,
            }
        })
        .set("test", TestGain::null(&geometry));

        assert_eq!(
            Some(AUTDInternalError::UnspecifiedKey("[\"null\"]".to_owned())),
            gain.calc(&geometry).err()
        );
    }

    #[test]
    fn test_calc_err() {
        let geometry = create_geometry(2, 249, FREQ_40K);

        let gain = Group::new(|_dev| |_tr| Some("test")).set("test", TestGain::err());

        assert_eq!(
            Some(AUTDInternalError::GainError("test".to_owned())),
            gain.calc(&geometry,).err()
        );
    }
}
