pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    geometry::{Device, Geometry, Transducer},
};
pub use autd3_derive::Gain;
use rayon::prelude::*;

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::Arc,
};

use bit_vec::BitVec;

use super::GainCalcResult;

pub trait GroupExec {
    type K: Hash + Eq + Clone + Debug;
    type FK: Fn(&Transducer) -> Option<Self::K>;
    type F: Fn(&Device) -> Self::FK;

    #[allow(clippy::type_complexity)]
    fn exec(
        gain_map: HashMap<Self::K, Vec<Box<dyn Fn(&Transducer) -> Drive + Send + Sync>>>,
        f: &Self::F,
        geometry: &Geometry,
        result: &[Vec<Drive>],
    ) -> Result<(), AUTDInternalError>;
}

pub struct Serial<K, FK, F>
where
    K: Hash + Eq + Clone + Debug,
    FK: Fn(&Transducer) -> Option<K>,
    F: Fn(&Device) -> FK,
{
    _phantom: std::marker::PhantomData<(K, FK, F)>,
}

impl<K, FK, F> GroupExec for Serial<K, FK, F>
where
    K: Hash + Eq + Clone + Debug,
    FK: Fn(&Transducer) -> Option<K>,
    F: Fn(&Device) -> FK,
{
    type K = K;
    type FK = FK;
    type F = F;

    fn exec(
        gain_map: HashMap<K, Vec<Box<dyn Fn(&Transducer) -> Drive + Send + Sync>>>,
        f: &F,
        geometry: &Geometry,
        result: &[Vec<Drive>],
    ) -> Result<(), AUTDInternalError> {
        gain_map
            .iter()
            // .par_iter()
            .try_for_each(|(k, g)| {
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
            })
    }
}

pub struct Parallel<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    _phantom: std::marker::PhantomData<(K, FK, F)>,
}

impl<K, FK, F> GroupExec for Parallel<K, FK, F>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    type K = K;
    type FK = FK;
    type F = F;

    fn exec(
        gain_map: HashMap<K, Vec<Box<dyn Fn(&Transducer) -> Drive + Send + Sync>>>,
        f: &F,
        geometry: &Geometry,
        result: &[Vec<Drive>],
    ) -> Result<(), AUTDInternalError> {
        gain_map.par_iter().try_for_each(|(k, g)| {
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
        })
    }
}

#[derive(Gain)]
pub struct Group<E: GroupExec> {
    f: E::F,
    gain_map: HashMap<E::K, Box<dyn Gain + Send + Sync>>,
}

impl<K, FK, F> Group<Serial<K, FK, F>>
where
    K: Hash + Eq + Clone + Debug,
    FK: Fn(&Transducer) -> Option<K>,
    F: Fn(&Device) -> FK,
{
    pub fn new(f: F) -> Self {
        Group {
            f,
            gain_map: HashMap::new(),
        }
    }
}

impl<K, FK, F> Group<Parallel<K, FK, F>>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    pub fn with_parallel(f: F) -> Self {
        Group {
            f,
            gain_map: HashMap::new(),
        }
    }
}

impl<E: GroupExec> Group<E> {
    pub fn set(mut self, key: E::K, gain: impl Gain + Send + Sync + 'static) -> Self {
        self.gain_map.insert(key, Box::new(gain));
        self
    }

    fn get_filters(&self, geometry: &Geometry) -> HashMap<E::K, HashMap<usize, BitVec<u32>>> {
        let mut filters: HashMap<E::K, HashMap<usize, BitVec<u32>>> = HashMap::new();
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

impl<E: GroupExec> Gain for Group<E> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        let mut filters = self.get_filters(geometry);

        let result = geometry
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
                let g = g.calc_with_filter(geometry, filter)?;
                Ok((
                    k.clone(),
                    geometry.devices().map(|dev| g(dev)).collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>, AUTDInternalError>>()?;

        let f = &self.f;
        E::exec(gain_map, f, geometry, &result)?;

        let drives_cache = geometry
            .devices()
            .zip(result)
            .map(|(dev, res)| (dev.idx(), Arc::new(res)))
            .collect::<HashMap<_, _>>();
        Ok(Box::new(move |dev| {
            let d = drives_cache[&dev.idx()].clone();
            Box::new(move |tr| d[tr.idx()])
        }))
    }

    #[tracing::instrument(level = "debug", skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("Group");
        if tracing::enabled!(tracing::Level::TRACE) {
            geometry.devices().for_each(|dev| {
                tracing::debug!(
                    "Device[{}]: {}",
                    dev.idx(),
                    dev.iter()
                        .map(|tr| { format!("{:#?}", (self.f)(dev)(tr)) })
                        .join(", ")
                );
            });
            self.gain_map.iter().for_each(|(k, g)| {
                tracing::debug!("Key: {:?}", k);
                Gain::trace(g, geometry);
            });
        }
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestGain, *};

    use crate::geometry::tests::create_geometry;

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(4, 249);

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
        .set("test", g1)
        .set("test2", g2);

        let g = gain.calc(&geometry)?;
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
    #[cfg_attr(miri, ignore)]
    fn with_parallel() -> anyhow::Result<()> {
        let geometry = create_geometry(4, 249);

        let mut rng = rand::thread_rng();

        let d1: Drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let d2: Drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let g1 = TestGain::new(|_| |_| d1, &geometry);
        let g2 = TestGain::new(|_| |_| d2, &geometry);

        let gain = Group::with_parallel(|dev| {
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

        let g = gain.calc(&geometry)?;
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
    #[cfg_attr(miri, ignore)]
    fn test_unknown_key() {
        let geometry = create_geometry(2, 249);

        let gain = Group::new(|_dev| {
            |tr| match tr.idx() {
                0..=99 => Some("test"),
                100..=199 => Some("null"),
                _ => None,
            }
        })
        .set("test2", TestGain::null(&geometry));

        assert_eq!(
            Some(AUTDInternalError::UnkownKey("\"test2\"".to_owned())),
            gain.calc(&geometry).err()
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_calc_err() {
        let geometry = create_geometry(2, 249);

        let gain = Group::new(|_dev| |_tr| Some("test")).set("test", TestGain::err());

        assert_eq!(
            Some(AUTDInternalError::GainError("test".to_owned())),
            gain.calc(&geometry,).err()
        );
    }
}
