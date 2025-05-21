use autd3_core::derive::*;

use autd3_driver::{
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};
use itertools::Itertools;

use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use derive_more::Debug;

/// [`Gain`] for grouping transducers and sending different [`Gain`] to each group.
///
/// If grouping by device is sufficient, [`autd3_driver::datagram::Group`] is recommended.
///
/// # Examples
///
/// ```
/// # use std::collections::HashMap;
/// use autd3::prelude::*;
/// use autd3::gain::{Group, IntoBoxedGain};
///
/// Group {
///     key_map: |dev| {
///         |tr| {
///             if tr.idx() < 100 {
///                 Some("null")
///             } else {
///                 Some("focus")
///             }
///         }
///     },
///     gain_map: HashMap::from([
///         ("null", Null {}.into_boxed()),
///         (
///             "focus",
///             Focus {
///                 pos: Point3::origin(),
///                 option: Default::default(),
///             }
///             .into_boxed(),
///         ),
///     ]),
/// };
/// ```
#[derive(Gain, Debug)]
pub struct Group<K, FK, F, G: Gain>
where
    K: Hash + Eq + std::fmt::Debug,
    FK: Fn(&Transducer) -> Option<K>,
    F: Fn(&Device) -> FK,
{
    /// Mapping function from transducer to group key.
    #[debug(ignore)]
    pub key_map: F,
    /// Map from group key to [`Gain`].
    #[debug(ignore)]
    pub gain_map: HashMap<K, G>,
}

impl<K, FK, F, G: Gain> Group<K, FK, F, G>
where
    K: Hash + Eq + std::fmt::Debug,
    FK: Fn(&Transducer) -> Option<K>,
    F: Fn(&Device) -> FK,
{
    /// Create a new [`Group`]
    #[must_use]
    pub const fn new(key_map: F, gain_map: HashMap<K, G>) -> Self {
        Self { key_map, gain_map }
    }

    #[must_use]
    fn get_filters(&self, geometry: &Geometry) -> HashMap<K, HashMap<usize, BitVec>> {
        let mut filters: HashMap<K, HashMap<usize, BitVec>> = HashMap::new();
        geometry.devices().for_each(|dev| {
            dev.iter().for_each(|tr| {
                if let Some(key) = (self.key_map)(dev)(tr) {
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

pub struct Impl {
    g: Vec<Drive>,
}

impl GainCalculator for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.g[tr.idx()]
    }
}

pub struct Generator {
    g: HashMap<usize, Vec<Drive>>,
}

impl GainCalculatorGenerator for Generator {
    type Calculator = Impl;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        Impl {
            g: self.g.remove(&device.idx()).unwrap(),
        }
    }
}

impl<K, FK, F, G: Gain> Gain for Group<K, FK, F, G>
where
    K: Hash + Eq + std::fmt::Debug,
    FK: Fn(&Transducer) -> Option<K>,
    F: Fn(&Device) -> FK,
{
    type G = Generator;

    fn init(
        self,
        geometry: &Geometry,
        _filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::G, GainError> {
        let filters = self.get_filters(geometry);

        let mut gain_map = self.gain_map;
        let gain_calcs = filters
            .into_iter()
            .map(|(k, filter)| {
                let g = gain_map
                    .remove(&k)
                    .ok_or(GainError::new(format!("Unknown group key: {:?}", k)))?;
                let mut g = g.init(geometry, Some(&filter))?;
                Ok((
                    k,
                    geometry
                        .iter()
                        .map(|dev| g.generate(dev))
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>, GainError>>()?;

        if !gain_map.is_empty() {
            return Err(GainError::new(format!(
                "Unused group keys: {}",
                gain_map.keys().map(|k| format!("{:?}", k)).join(", "),
            )));
        }

        let f = &self.key_map;
        Ok(Self::G {
            g: geometry
                .devices()
                .map(|dev| {
                    let f = (f)(dev);
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|tr| {
                                if let Some(key) = f(tr) {
                                    gain_calcs[&key][dev.idx()].calc(tr)
                                } else {
                                    Drive::NULL
                                }
                            })
                            .collect(),
                    )
                })
                .collect(),
        })
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

        let mut rng = rand::rng();

        let d1 = Drive {
            phase: Phase(rng.random()),
            intensity: EmitIntensity(rng.random()),
        };
        let d2 = Drive {
            phase: Phase(rng.random()),
            intensity: EmitIntensity(rng.random()),
        };

        let gain = Group::new(
            |dev| {
                let dev_idx = dev.idx();
                move |tr| match (dev_idx, tr.idx()) {
                    (0, 0..=99) => Some("null"),
                    (0, 100..=199) => Some("test"),
                    (1, 200..) => Some("test2"),
                    (3, _) => Some("test"),
                    _ => None,
                }
            },
            HashMap::from([
                ("null", Null {}.into_boxed()),
                (
                    "test",
                    Uniform {
                        intensity: d1.intensity,
                        phase: d1.phase,
                    }
                    .into_boxed(),
                ),
                (
                    "test2",
                    Uniform {
                        intensity: d2.intensity,
                        phase: d2.phase,
                    }
                    .into_boxed(),
                ),
            ]),
        );

        let mut g = gain.init(&geometry, None)?;
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
    fn unknown_key() -> anyhow::Result<()> {
        let gain = Group {
            key_map: |_dev| |_tr| Some("test"),
            gain_map: HashMap::<_, Null>::new(),
        };
        let geometry = create_geometry(1);
        assert_eq!(
            Some(GainError::new("Unknown group key: \"test\"")),
            gain.init(&geometry, None).err()
        );

        Ok(())
    }

    #[test]
    fn unused_key() -> anyhow::Result<()> {
        let gain = Group {
            key_map: |_dev| |_tr| Some(1),
            gain_map: HashMap::from([(1, Null {}), (2, Null {})]),
        };

        let geometry = create_geometry(1);
        assert_eq!(
            Some(GainError::new("Unused group keys: 2")),
            gain.init(&geometry, None).err()
        );

        Ok(())
    }
}
