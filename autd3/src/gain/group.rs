/*
 * File: mod.rs
 * Project: group
 * Created Date: 18/08/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::{collections::HashMap, hash::Hash};

use bitvec::prelude::*;

use autd3_driver::{
    common::EmitIntensity,
    derive::prelude::*,
    geometry::{Device, Geometry},
};

pub struct Group<K: Hash + Eq + Clone, G: Gain, F: Fn(&Device, &Transducer) -> Option<K>> {
    f: F,
    gain_map: HashMap<K, G>,
}

impl<K: Hash + Eq + Clone, F: Fn(&Device, &Transducer) -> Option<K>> Group<K, Box<dyn Gain>, F> {
    /// Group by transducer
    ///
    /// # Arguments
    /// `f` - function to get key from transducer (currentry, transducer type annotation is required)
    ///
    /// # Exintensityles
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # let gain : autd3::gain::Group<_, _, _> =
    /// Group::new(|dev, tr| match tr.idx() {
    ///                 0..=100 => Some("null"),
    ///                 101.. => Some("focus"),
    ///                 _ => None,
    ///             })
    ///             .set("null", Null::new())
    ///             .set("focus", Focus::new(Vector3::new(0.0, 0.0, 150.0)));
    /// ```
    pub fn new(f: F) -> Group<K, Box<dyn Gain>, F> {
        Group {
            f,
            gain_map: HashMap::new(),
        }
    }
}

impl<K: Hash + Eq + Clone, G: Gain, F: Fn(&Device, &Transducer) -> Option<K>> Group<K, G, F> {
    /// get gain map which maps device id to gain
    pub fn gain_map(&self) -> &HashMap<K, G> {
        &self.gain_map
    }
}

impl<'a, K: Hash + Eq + Clone, F: Fn(&Device, &Transducer) -> Option<K>>
    Group<K, Box<dyn Gain + 'a>, F>
{
    /// set gain
    ///
    /// # Arguments
    ///
    /// * `key` - key
    /// * `gain` - Gain
    ///
    pub fn set<G: Gain + 'a>(mut self, key: K, gain: G) -> Self {
        self.gain_map.insert(key, Box::new(gain));
        self
    }
}

impl<K: Hash + Eq + Clone, F: Fn(&Device, &Transducer) -> Option<K>> Group<K, Box<dyn Gain>, F> {
    /// get Gain of specified key
    ///
    /// # Arguments
    ///
    /// * `key` - key
    ///
    /// # Returns
    ///
    /// * Gain of specified key if exists and the type is matched, otherwise None
    ///
    pub fn get<G: Gain + 'static>(&self, key: K) -> Option<&G> {
        self.gain_map
            .get(&key)
            .and_then(|g| g.as_ref().as_any().downcast_ref::<G>())
    }
}

impl<
        K: Hash + Eq + Clone + 'static,
        G: Gain + 'static,
        F: Fn(&Device, &Transducer) -> Option<K> + 'static,
    > autd3_driver::datagram::Datagram for Group<K, G, F>
{
    type O1 = autd3_driver::operation::GainOp<Self>;
    type O2 = autd3_driver::operation::NullOp;

    fn operation(self) -> Result<(Self::O1, Self::O2), autd3_driver::error::AUTDInternalError> {
        Ok((Self::O1::new(self), Self::O2::default()))
    }
}

impl<
        K: Hash + Eq + Clone + 'static,
        G: Gain + 'static,
        F: Fn(&Device, &Transducer) -> Option<K> + 'static,
    > GainAsAny for Group<K, G, F>
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl<
        K: Hash + Eq + Clone + 'static,
        G: Gain + 'static,
        F: Fn(&Device, &Transducer) -> Option<K> + 'static,
    > Group<K, G, F>
{
    fn get_filters(&self, geometry: &Geometry) -> HashMap<K, HashMap<usize, BitVec<usize, Lsb0>>> {
        let mut filters = HashMap::new();
        geometry.devices().for_each(|dev| {
            dev.iter().for_each(|tr| {
                if let Some(key) = (self.f)(dev, tr) {
                    if !filters.contains_key(&key) {
                        let mut filter = BitVec::<usize, Lsb0>::new();
                        filter.resize(dev.num_transducers(), false);
                        let filter: HashMap<usize, BitVec<usize, Lsb0>> =
                            [(dev.idx(), filter)].into();
                        filters.insert(key.clone(), filter);
                    }
                    filters
                        .get_mut(&key)
                        .unwrap()
                        .entry(dev.idx())
                        .or_insert_with(|| {
                            let mut filter = BitVec::<usize, Lsb0>::new();
                            filter.resize(dev.num_transducers(), false);
                            filter
                        });
                    filters
                        .get_mut(&key)
                        .unwrap()
                        .get_mut(&dev.idx())
                        .unwrap()
                        .set(tr.idx(), true);
                }
            })
        });
        filters
    }
}

impl<
        K: Hash + Eq + Clone + 'static,
        G: Gain + 'static,
        F: Fn(&Device, &Transducer) -> Option<K> + 'static,
    > Gain for Group<K, G, F>
{
    #[allow(clippy::uninit_vec)]
    fn calc(
        &self,
        geometry: &Geometry,
        _filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let filters = self.get_filters(geometry);

        let drives_cache = self
            .gain_map
            .iter()
            .map(|(k, g)| {
                let k = k.clone();
                let filter = if let Some(f) = filters.get(&k) {
                    f
                } else {
                    return Err(AUTDInternalError::GainError("Unknown group key".to_owned()));
                };
                let d = g.calc(geometry, GainFilter::Filter(filter))?;
                Ok((k, d))
            })
            .collect::<Result<HashMap<_, HashMap<usize, Vec<Drive>>>, _>>()?;

        geometry
            .devices()
            .map(|dev| {
                let mut d: Vec<Drive> = Vec::with_capacity(dev.num_transducers());
                unsafe {
                    d.set_len(dev.num_transducers());
                }
                for tr in dev.iter() {
                    if let Some(key) = (self.f)(dev, tr) {
                        let g = if let Some(g) = drives_cache.get(&key) {
                            g
                        } else {
                            return Err(AUTDInternalError::GainError(
                                "Unspecified group key".to_owned(),
                            ));
                        };
                        d[tr.idx()] = g[&dev.idx()][tr.idx()];
                    } else {
                        d[tr.idx()] = Drive {
                            intensity: EmitIntensity::MIN,
                            phase: Phase::new(0),
                        }
                    }
                }
                Ok((dev.idx(), d))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{IntoDevice, Transducer, Vector3},
    };

    use super::*;

    use crate::gain::{Focus, Null, Plane};

    #[test]
    fn test_group() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
            AUTD3::new(Vector3::zeros()).into_device(2),
            AUTD3::new(Vector3::zeros()).into_device(3),
        ]);

        let gain = Group::new(|dev, tr: &Transducer| match (dev.idx(), tr.idx()) {
            (0, 0..=99) => Some("null"),
            (0, 100..=199) => Some("plane"),
            (1, 200..) => Some("plane2"),
            _ => None,
        })
        .set("null", Null::new())
        .set("plane", Plane::new(Vector3::zeros()))
        .set("plane2", Plane::new(Vector3::zeros()).with_intensity(0x1F));

        let drives = gain.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(drives.len(), 4);
        assert!(drives.values().all(|d| d.len() == AUTD3::NUM_TRANS_IN_UNIT));

        drives[&0].iter().enumerate().for_each(|(i, d)| match i {
            i if i <= 99 => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0);
            }
            i if i <= 199 => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0xFF);
            }
            _ => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0);
            }
        });
        drives[&1].iter().enumerate().for_each(|(i, d)| match i {
            i if i <= 199 => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0);
            }
            _ => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0x1F);
            }
        });
        drives[&2].iter().for_each(|d| {
            assert_eq!(d.phase.value(), 0);
            assert_eq!(d.intensity.value(), 0);
        });
        drives[&3].iter().for_each(|d| {
            assert_eq!(d.phase.value(), 0);
            assert_eq!(d.intensity.value(), 0);
        });
    }

    #[test]
    fn test_group_unknown_key() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
        ]);

        let gain = Group::new(|_dev, tr: &Transducer| match tr.idx() {
            0..=99 => Some("plane"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("plane2", Plane::new(Vector3::zeros()));

        match gain.calc(&geometry, GainFilter::All) {
            Ok(_) => panic!("Should be error"),
            Err(e) => assert_eq!(
                e,
                AUTDInternalError::GainError("Unknown group key".to_owned())
            ),
        }
    }

    #[test]
    fn test_group_unspecified_key() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
        ]);

        let gain = Group::new(|_dev, tr: &Transducer| match tr.idx() {
            0..=99 => Some("plane"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("plane", Plane::new(Vector3::zeros()));

        match gain.calc(&geometry, GainFilter::All) {
            Ok(_) => panic!("Should be error"),
            Err(e) => assert_eq!(
                e,
                AUTDInternalError::GainError("Unspecified group key".to_owned())
            ),
        }
    }

    #[test]
    fn test_get() {
        let gain: Group<_, _, _> = Group::new(|dev, _tr| match dev.idx() {
            0 => Some("null"),
            1 => Some("plane"),
            2 | 3 => Some("plane2"),
            _ => None,
        })
        .set("null", Null::new())
        .set("plane", Plane::new(Vector3::zeros()))
        .set("plane2", Plane::new(Vector3::zeros()).with_intensity(0x1F));

        assert!(gain.get::<Null>("null").is_some());
        assert!(gain.get::<Focus>("null").is_none());

        assert!(gain.get::<Plane>("plane").is_some());
        assert!(gain.get::<Null>("plane").is_none());
        assert_eq!(
            gain.get::<Plane>("plane").unwrap().intensity().value(),
            0xFF
        );

        assert!(gain.get::<Plane>("plane2").is_some());
        assert!(gain.get::<Null>("plane2").is_none());
        assert_eq!(
            gain.get::<Plane>("plane2").unwrap().intensity().value(),
            0x1F
        );

        assert!(gain.get::<Null>("focus").is_none());
        assert!(gain.get::<Focus>("focus").is_none());
        assert!(gain.get::<Plane>("focus").is_none());
    }
}
