use autd3_core::derive::*;

use autd3_driver::{
    error::AUTDDriverError,
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};
use itertools::Itertools;
use rayon::prelude::*;

use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use derive_more::Debug;
use derive_new::new;

/// [`Gain`] for grouping transducers and sending different [`Gain`] to each group.
///
/// If grouping by device is sufficient, [`Controller::group`] is recommended.
///
/// # Examples
///
/// ```
/// use autd3::prelude::*;
///
/// # fn _main() -> Result<(), AUTDDriverError> {
/// Group::new(|dev| |tr| if tr.idx() < 100 { Some("null") } else { Some("focus") })
///    .set("null", Null {}.into_boxed())?
///    .set("focus", Focus { pos: Point3::origin(), option: Default::default() }.into_boxed())?;
/// # Ok(())
/// # }
/// ```
///
/// [`Controller::group`]: crate::controller::Controller::group
#[derive(Gain, Debug, new)]
pub struct Group<K, FK, F, G: Gain>
where
    K: Hash + Eq + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    #[debug(ignore)]
    f: F,
    #[new(default)]
    gain_map: HashMap<K, G>,
}

impl<K, FK, F, G: Gain> Group<K, FK, F, G>
where
    K: Hash + Eq + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    /// Set the [`Gain`] to the transducers corresponding to the `key`.
    ///
    /// # Errors
    ///
    /// Returns [`AUTDDriverError::KeyIsAlreadyUsed`] if the `key` is already used previous [`Group::set`].
    #[allow(clippy::map_entry)] // https://github.com/rust-lang/rust-clippy/issues/9925
    pub fn set(mut self, key: K, gain: G) -> Result<Self, AUTDDriverError> {
        if self.gain_map.contains_key(&key) {
            return Err(AUTDDriverError::KeyIsAlreadyUsed(format!("{:?}", key)));
        } else {
            self.gain_map.insert(key, gain);
        }
        Ok(self)
    }

    fn get_filters(&self, geometry: &Geometry) -> HashMap<K, HashMap<usize, BitVec>> {
        let mut filters: HashMap<K, HashMap<usize, BitVec>> = HashMap::new();
        geometry.devices().for_each(|dev| {
            dev.iter().for_each(|tr| {
                if let Some(key) = (self.f)(dev)(tr) {
                    if let Some(v) = filters.get_mut(&key) {
                        match v.entry(dev.idx()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().set(tr.idx(), true);
                            }
                            Entry::Vacant(e) => {
                                e.insert(BitVec::from_fn(dev.num_transducers(), |i| i == tr.idx()));
                            }
                        }
                    } else {
                        filters.insert(
                            key,
                            [(
                                dev.idx(),
                                BitVec::from_fn(dev.num_transducers(), |i| i == tr.idx()),
                            )]
                            .into(),
                        );
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

impl<K, FK, F, G: Gain> Gain for Group<K, FK, F, G>
where
    K: Hash + Eq + Debug + Send + Sync,
    FK: Fn(&Transducer) -> Option<K> + Send + Sync,
    F: Fn(&Device) -> FK + Send + Sync,
{
    type G = ContextGenerator;

    // GRCOV_EXCL_START
    fn init(self) -> Result<Self::G, GainError> {
        unimplemented!()
    }
    // GRCOV_EXCL_STOP

    fn init_full(
        self,
        geometry: &Geometry,
        _filter: Option<&HashMap<usize, BitVec>>,
        option: &DatagramOption,
    ) -> Result<Self::G, GainError> {
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
                    .ok_or(GainError::new(format!("Unknown group key({:?})", k)))?;
                let mut g = g.init_full(geometry, Some(&filter), option)?;
                Ok((
                    k,
                    geometry
                        .devices()
                        .map(|dev| g.generate(dev))
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>, GainError>>()?;

        if !filters.is_empty() {
            return Err(GainError::new(format!(
                "Unused group keys: {}",
                filters.keys().map(|k| format!("{:?}", k)).join(", "),
            )));
        }

        let f = &self.f;
        if geometry.num_devices() > option.parallel_threshold {
            gain_map
                .par_iter()
                .try_for_each(|(k, c)| -> Result<(), GainError> {
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
                .try_for_each(|(k, c)| -> Result<(), GainError> {
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

    use autd3_driver::{
        datagram::IntoBoxedGain,
        firmware::fpga::{EmitIntensity, Phase},
    };
    use rand::Rng;

    use crate::{
        gain::{Null, Uniform},
        tests::create_geometry,
    };

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(4);

        let mut rng = rand::thread_rng();

        let d1 = Drive {
            phase: Phase(rng.gen()),
            intensity: EmitIntensity(rng.gen()),
        };
        let d2 = Drive {
            phase: Phase(rng.gen()),
            intensity: EmitIntensity(rng.gen()),
        };

        let g1 = Uniform {
            intensity: d1.intensity,
            phase: d1.phase,
        };
        let g2 = Uniform {
            intensity: d2.intensity,
            phase: d2.phase,
        };

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
        .set("null", Null {}.into_boxed())?
        .set("test", g1.into_boxed())?
        .set("test2", g2.into_boxed())?;

        let mut g = gain.init_full(&geometry, None, &DatagramOption::default())?;
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
        let geometry = create_geometry(5);

        let mut rng = rand::thread_rng();

        let d1 = Drive {
            phase: Phase(rng.gen()),
            intensity: EmitIntensity(rng.gen()),
        };
        let d2 = Drive {
            phase: Phase(rng.gen()),
            intensity: EmitIntensity(rng.gen()),
        };

        let g1 = Uniform {
            intensity: d1.intensity,
            phase: d1.phase,
        };
        let g2 = Uniform {
            intensity: d2.intensity,
            phase: d2.phase,
        };

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
        .set("null", Null {}.into_boxed())?
        .set("test", g1.into_boxed())?
        .set("test2", g2.into_boxed())?;

        let mut g = gain.init_full(
            &geometry,
            None,
            &DatagramOption {
                parallel_threshold: 4,
                ..Default::default()
            },
        )?;
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
        assert_eq!(5, drives.len());
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
        drives[&4].iter().for_each(|&d| {
            assert_eq!(Drive::NULL, d);
        });

        Ok(())
    }

    #[test]
    fn unknown_key() -> anyhow::Result<()> {
        let gain = Group::new(|_dev| {
            |tr| match tr.idx() {
                0..=99 => Some("test"),
                100..=199 => Some("null"),
                _ => None,
            }
        })
        .set("test2", Null {})?;

        let geometry = create_geometry(1);
        assert_eq!(
            Some(GainError::new("Unknown group key(\"test2\")".to_owned())),
            gain.init_full(&geometry, None, &DatagramOption::default())
                .err()
        );

        Ok(())
    }

    #[test]
    fn already_used_key() -> anyhow::Result<()> {
        let gain = Group::new(|_dev| |_tr| Some(0))
            .set(0, Null {})?
            .set(0, Null {});

        assert_eq!(
            Some(AUTDDriverError::KeyIsAlreadyUsed("0".to_owned())),
            gain.err()
        );

        Ok(())
    }

    #[test]
    fn unused_key() -> anyhow::Result<()> {
        let gain = Group::new(|_dev| {
            |tr| match tr.idx() {
                0..=99 => Some(0),
                _ => Some(1),
            }
        })
        .set(1, Null {})?;

        let geometry = create_geometry(1);
        assert_eq!(
            Some(GainError::new("Unused group keys: 0".to_owned())),
            gain.init_full(&geometry, None, &DatagramOption::default())
                .err()
        );

        Ok(())
    }
}
