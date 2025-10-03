use autd3_core::derive::*;

use autd3_driver::geometry::{Device, Transducer};

use itertools::Itertools;
use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

/// [`Gain`] for grouping transducers and sending different [`Gain`] to each group.
///
/// If grouping by device is sufficient, [`autd3_driver::datagram::Group`] is recommended.
///
/// # Examples
///
/// ```
/// # use std::collections::HashMap;
/// use autd3::prelude::*;
/// use autd3::gain::Group;
///
/// Group::new(
///     |dev| {
///         |tr| {
///             if tr.idx() < 100 {
///                 Some("null")
///             } else {
///                 Some("focus")
///             }
///         }
///     },
///     HashMap::from([
///         ("null", BoxedGain::new(Null {})),
///         (
///             "focus",
///             BoxedGain::new(Focus {
///                 pos: Point3::origin(),
///                 option: Default::default(),
///             }),
///         ),
///     ]),
/// );
/// ```
#[derive(Gain, Debug)]
pub struct Group<'geo, K, FK, F, G>
where
    K: Hash + Eq + std::fmt::Debug,
    FK: Fn(&'geo Transducer) -> Option<K>,
    F: Fn(&'geo Device) -> FK,
{
    /// Mapping function from transducer to group key.
    pub key_map: F,
    /// Map from group key to [`Gain`].
    pub gain_map: HashMap<K, G>,
    _phantom: std::marker::PhantomData<(&'geo (), &'geo ())>,
}

impl<'a, K, FK, F, G: Gain<'a>> Group<'a, K, FK, F, G>
where
    K: Hash + Eq + std::fmt::Debug,
    FK: Fn(&'a Transducer) -> Option<K>,
    F: Fn(&'a Device) -> FK,
{
    /// Create a new [`Group`]
    #[must_use]
    pub const fn new(key_map: F, gain_map: HashMap<K, G>) -> Self {
        Self {
            key_map,
            gain_map,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    fn get_filters(
        &self,
        geometry: &'a Geometry,
        tr_filter: &TransducerMask,
    ) -> HashMap<K, TransducerMask> {
        let mut filters: HashMap<K, HashMap<usize, Vec<bool>>> = HashMap::new();
        geometry
            .iter()
            .filter(|dev| tr_filter.has_enabled(dev))
            .for_each(|dev| {
                dev.iter().for_each(|tr| {
                    if let Some(key) = (self.key_map)(dev)(tr) {
                        if let Some(v) = filters.get_mut(&key) {
                            match v.entry(dev.idx()) {
                                Entry::Occupied(mut e) => {
                                    e.get_mut()[tr.idx()] = true;
                                }
                                Entry::Vacant(e) => {
                                    e.insert(
                                        (0..dev.num_transducers()).map(|t| t == tr.idx()).collect(),
                                    );
                                }
                            }
                        } else {
                            filters.insert(
                                key,
                                [(
                                    dev.idx(),
                                    (0..dev.num_transducers()).map(|t| t == tr.idx()).collect(),
                                )]
                                .into(),
                            );
                        }
                    }
                })
            });
        filters
            .into_iter()
            .map(|(k, mut v)| {
                (
                    k,
                    TransducerMask::new(geometry.iter().map(|dev| {
                        if let Some(mask) = v.remove(&dev.idx()) {
                            DeviceTransducerMask::Masked(mask)
                        } else {
                            DeviceTransducerMask::AllDisabled
                        }
                    })),
                )
            })
            .collect()
    }
}

pub struct Impl {
    g: Vec<Drive>,
}

impl GainCalculator<'_> for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.g[tr.idx()]
    }
}

pub struct Generator {
    g: HashMap<usize, Vec<Drive>>,
}

impl GainCalculatorGenerator<'_> for Generator {
    type Calculator = Impl;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        Impl {
            g: self.g.remove(&device.idx()).unwrap(),
        }
    }
}

impl<'a, K, FK, F, G: Gain<'a>> Gain<'a> for Group<'a, K, FK, F, G>
where
    K: Hash + Eq + std::fmt::Debug,
    FK: Fn(&'a Transducer) -> Option<K>,
    F: Fn(&'a Device) -> FK,
{
    type G = Generator;

    fn init(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        device_filter: &TransducerMask,
    ) -> Result<Self::G, GainError> {
        let filters = self.get_filters(geometry, device_filter);

        let mut gain_map = self.gain_map;
        let gain_calcs = filters
            .into_iter()
            .map(|(k, filter)| {
                let g = gain_map
                    .remove(&k)
                    .ok_or(GainError::new(format!("Unknown group key: {k:?}")))?;
                let mut g = g.init(geometry, env, &filter)?;
                Ok((
                    k,
                    geometry
                        .iter()
                        .map(|dev| filter.has_enabled(dev).then(|| g.generate(dev)))
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>, GainError>>()?;

        if !gain_map.is_empty() {
            return Err(GainError::new(format!(
                "Unused group keys: {}",
                gain_map.keys().map(|k| format!("{k:?}")).join(", "),
            )));
        }

        let f = &self.key_map;
        Ok(Self::G {
            g: geometry
                .iter()
                .filter(|dev| device_filter.has_enabled(dev))
                .map(|dev| {
                    let f = (f)(dev);
                    (
                        dev.idx(),
                        dev.iter()
                            .map(|tr| {
                                if let Some(key) = f(tr) {
                                    gain_calcs[&key][dev.idx()].as_ref().unwrap().calc(tr)
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

    use autd3_driver::datagram::BoxedGain;
    use rand::Rng;

    use crate::{
        gain::{Null, Uniform},
        tests::create_geometry,
    };

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = create_geometry(4);
        let env = Environment::new();

        let mut rng = rand::rng();

        let d1 = Drive {
            phase: Phase(rng.random()),
            intensity: Intensity(rng.random()),
        };
        let d2 = Drive {
            phase: Phase(rng.random()),
            intensity: Intensity(rng.random()),
        };

        let gain = Group::new(
            |dev| {
                move |tr| match (dev.idx(), tr.idx()) {
                    (0, 0..=99) => Some("null"),
                    (0, 100..=199) => Some("test"),
                    (1, 200..) => Some("test2"),
                    (3, _) => Some("test"),
                    _ => None,
                }
            },
            HashMap::from([
                ("null", BoxedGain::new(Null {})),
                (
                    "test",
                    BoxedGain::new(Uniform {
                        intensity: d1.intensity,
                        phase: d1.phase,
                    }),
                ),
                (
                    "test2",
                    BoxedGain::new(Uniform {
                        intensity: d2.intensity,
                        phase: d2.phase,
                    }),
                ),
            ]),
        );

        let mut g = gain.init(&geometry, &env, &TransducerMask::AllEnabled)?;
        let drives = geometry
            .iter()
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
    fn unknown_key() -> Result<(), Box<dyn std::error::Error>> {
        let gain = Group::new(|_dev| |_tr| Some("test"), HashMap::<_, Null>::new());
        let geometry = create_geometry(1);
        let env = Environment::new();
        assert_eq!(
            Some(GainError::new("Unknown group key: \"test\"")),
            gain.init(&geometry, &env, &TransducerMask::AllEnabled)
                .err()
        );

        Ok(())
    }

    #[test]
    fn unused_key() -> Result<(), Box<dyn std::error::Error>> {
        let gain = Group::new(
            |_dev| |_tr| Some(1),
            HashMap::from([(1, Null {}), (2, Null {})]),
        );

        let geometry = create_geometry(1);
        let env = Environment::new();
        assert_eq!(
            Some(GainError::new("Unused group keys: 2")),
            gain.init(&geometry, &env, &TransducerMask::AllEnabled)
                .err()
        );

        Ok(())
    }
}
