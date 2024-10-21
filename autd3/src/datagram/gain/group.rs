use autd3_driver::{
    datagram::{BoxedGain, IntoBoxedGain},
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

use derive_more::Debug;
use derive_new::new;

#[derive(Gain, Builder, Debug, new)]
pub struct Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    #[debug(ignore)]
    f: F,
    #[new(default)]
    gain_map: HashMap<K, BoxedGain>,
    #[new(default)]
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
    pub fn set(mut self, key: K, gain: impl IntoBoxedGain) -> Self {
        self.gain_map.insert(key, gain.into_boxed());
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
                                e.insert(BitVec::from_fn(dev.num_transducers(), |i| i == tr.idx()));
                            }
                        },
                        None => {
                            filters.insert(
                                key.clone(),
                                [(
                                    dev.idx(),
                                    BitVec::from_fn(dev.num_transducers(), |i| i == tr.idx()),
                                )]
                                .into(),
                            );
                        }
                    }
                }
            })
        });
        filters
    }
}

pub struct Context {
    g: Vec<Drive>,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.g[tr.idx()]
    }
}

pub struct ContextGenerator {
    g: HashMap<usize, Vec<Drive>>,
}

impl GainContextGenerator for ContextGenerator {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            g: self.g.remove(&device.idx()).unwrap(),
        }
    }
}

impl<K, FK, F> Gain for Group<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    type G = ContextGenerator;

    fn init_with_filter(
        self,
        geometry: &Geometry,
        _filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        let mut filters = self.get_filters(geometry);

        let mut g = geometry
            .devices()
            .map(|dev| {
                (
                    dev.idx(),
                    dev.iter().map(|_| Drive::NULL).collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, Vec<_>>>();
        let gain_map = self
            .gain_map
            .into_iter()
            .map(|(k, g)| {
                let filter = filters
                    .remove(&k)
                    .ok_or(AUTDInternalError::UnkownKey(format!("{:?}", k)))?;
                let mut g = g.init_with_filter(geometry, Some(filter))?;
                Ok((
                    k.clone(),
                    geometry
                        .devices()
                        .map(|dev| g.generate(dev))
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>, AUTDInternalError>>()?;

        let f = &self.f;
        if self.parallel {
            gain_map
                .par_iter()
                .try_for_each(|(k, c)| -> Result<(), AUTDInternalError> {
                    geometry.devices().zip(c.iter()).for_each(|(dev, c)| {
                        let f = (f)(dev);
                        let r = g[&dev.idx()].as_ptr() as *mut Drive;
                        dev.iter()
                            .filter(|tr| f(tr).is_some_and(|kk| &kk == k))
                            .for_each(|tr| unsafe {
                                r.add(tr.idx()).write(c.calc(tr));
                            })
                    });
                    Ok(())
                })?;
        } else {
            gain_map
                .iter()
                .try_for_each(|(k, c)| -> Result<(), AUTDInternalError> {
                    geometry.devices().zip(c.iter()).for_each(|(dev, c)| {
                        let f = (f)(dev);
                        let r = g.get_mut(&dev.idx()).unwrap();
                        dev.iter()
                            .filter(|tr| f(tr).is_some_and(|kk| &kk == k))
                            .for_each(|tr| {
                                r[tr.idx()] = c.calc(tr);
                            })
                    });
                    Ok(())
                })?;
        }

        Ok(Self::G { g })
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

        let mut g = gain.init(&geometry)?;
        let drives = geometry
            .devices()
            .map(|dev| {
                let f = g.generate(dev);
                (
                    dev.idx(),
                    dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();
        assert_eq!(4, drives.len());
        drives[&0].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 99 => {
                assert_eq!(Drive::NULL, d);
            }
            i if i <= 199 => {
                assert_eq!(d1, d);
            }
            _ => {
                assert_eq!(Drive::NULL, d);
            }
        });
        drives[&1].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 199 => {
                assert_eq!(Drive::NULL, d);
            }
            _ => {
                assert_eq!(d2, d);
            }
        });
        drives[&2].iter().for_each(|&d| {
            assert_eq!(Drive::NULL, d);
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

        let mut g = gain.init(&geometry)?;
        let drives = geometry
            .devices()
            .map(|dev| {
                let f = g.generate(dev);
                (
                    dev.idx(),
                    dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();
        assert_eq!(4, drives.len());
        drives[&0].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 99 => {
                assert_eq!(Drive::NULL, d);
            }
            i if i <= 199 => {
                assert_eq!(d1, d);
            }
            _ => {
                assert_eq!(Drive::NULL, d);
            }
        });
        drives[&1].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 199 => {
                assert_eq!(Drive::NULL, d);
            }
            _ => {
                assert_eq!(d2, d);
            }
        });
        drives[&2].iter().for_each(|&d| {
            assert_eq!(Drive::NULL, d);
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
            gain.init(&geometry).err()
        );
    }
}
