use autd3_driver::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};
use rayon::prelude::*;

use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use bit_vec::BitVec;

use derive_more::Debug;

#[derive(Gain, Builder, Debug)]
pub struct Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    #[debug(ignore)]
    f: F,
    gain_map: HashMap<K, Box<dyn Gain + Send + Sync>>,
    #[get]
    #[set]
    parallel: bool,
}

impl<K, FK, F> Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    pub fn new(f: F) -> Self {
        Group {
            f,
            gain_map: HashMap::new(),
            parallel: false,
        }
    }
}

impl<K, FK, F> Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    pub fn set(mut self, key: K, gain: impl Gain + Send + Sync + 'static) -> Self {
        self.gain_map.insert(key, Box::new(gain));
        self
    }

    fn get_filters(&self, geometry: &Geometry) -> HashMap<K, HashMap<usize, BitVec<u32>>> {
        let mut filters: HashMap<K, HashMap<usize, BitVec<u32>>> = HashMap::new();
        geometry.devices().for_each(|dev| {
            dev.iter().for_each(|tr| {
                if let Some(key) = (self.f)(dev)(tr) {
                    match filters.get_mut(&key) {
                        Some(v) => match v.entry(dev.idx()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().set(tr.idx(), true);
                            }
                            Entry::Vacant(e) => {
                                let mut filter = BitVec::from_elem(dev.num_transducers(), false);
                                filter.set(tr.idx(), true);
                                e.insert(filter);
                            }
                        },
                        None => {
                            let mut filter = BitVec::from_elem(dev.num_transducers(), false);
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
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    fn calc(&self, geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
        let mut filters = self.get_filters(geometry);

        let mut result = geometry
            .devices()
            .map(|dev| dev.iter().map(|_| Drive::null()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let gain_map = self
            .gain_map
            .iter()
            .map(|(k, g)| {
                let filter = filters
                    .remove(k)
                    .ok_or(AUTDInternalError::UnkownKey(format!("{:?}", k)))?;
                let mut g = g.calc_with_filter(geometry, filter)?;
                Ok((
                    k.clone(),
                    geometry.devices().map(|dev| g(dev)).collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>, AUTDInternalError>>()?;

        let f = &self.f;
        if self.parallel {
            gain_map
                .par_iter()
                .try_for_each(|(k, g)| -> Result<(), AUTDInternalError> {
                    geometry
                        .devices()
                        .zip(g.iter())
                        .zip(result.iter())
                        .for_each(|((dev, g), result)| {
                            let f = (f)(dev);
                            let r = result.as_ptr() as *mut Drive;
                            dev.iter().for_each(|tr| {
                                if let Some(kk) = f(tr) {
                                    if &kk == k {
                                        unsafe {
                                            r.add(tr.idx()).write(g(tr));
                                        }
                                    }
                                }
                            })
                        });
                    Ok(())
                })?;
        } else {
            gain_map
                .iter()
                .try_for_each(|(k, g)| -> Result<(), AUTDInternalError> {
                    geometry
                        .devices()
                        .zip(g.iter())
                        .zip(result.iter())
                        .for_each(|((dev, g), result)| {
                            let f = (f)(dev);
                            let r = result.as_ptr() as *mut Drive;
                            dev.iter().for_each(|tr| {
                                if let Some(kk) = f(tr) {
                                    if &kk == k {
                                        unsafe {
                                            r.add(tr.idx()).write(g(tr));
                                        }
                                    }
                                }
                            })
                        });
                    Ok(())
                })?;
        }

        Ok(Box::new(move |dev| {
            let mut tmp = vec![];
            std::mem::swap(&mut tmp, &mut result[dev.idx()]);
            Box::new(move |tr| tmp[tr.idx()])
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;

    use crate::{
        gain::{Null, Uniform},
        tests::create_geometry,
    };

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(4);

        let mut rng = rand::thread_rng();

        let d1 = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let d2 = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let g1 = Uniform::new(d1);
        let g2 = Uniform::new(d2);

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
        .set("test", g1)
        .set("test2", g2);

        let mut g = gain.calc(&geometry)?;
        let drives = geometry
            .devices()
            .map(|dev| (dev.idx(), dev.iter().map(g(dev)).collect::<Vec<_>>()))
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
    fn with_parallel() -> anyhow::Result<()> {
        let geometry = create_geometry(4);

        let mut rng = rand::thread_rng();

        let d1 = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let d2 = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let g1 = Uniform::new(d1);
        let g2 = Uniform::new(d2);

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
        .with_parallel(true)
        .set("test", g1)
        .set("test2", g2);

        let mut g = gain.calc(&geometry)?;
        let drives = geometry
            .devices()
            .map(|dev| (dev.idx(), dev.iter().map(g(dev)).collect::<Vec<_>>()))
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
        let geometry = create_geometry(2);

        let gain = Group::new(|_dev| {
            |tr| match tr.idx() {
                0..=99 => Some("test"),
                100..=199 => Some("null"),
                _ => None,
            }
        })
        .set("test2", Null::new());

        assert_eq!(
            Some(AUTDInternalError::UnkownKey("\"test2\"".to_owned())),
            gain.calc(&geometry).err()
        );
    }
}
